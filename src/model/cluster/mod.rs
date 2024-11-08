use std::fs::File;
use std::{future::ready, sync::Arc};

use anyhow::Context;
use futures_util::StreamExt as _;
use kube::runtime::{reflector::ObjectRef, WatchStreamExt as _};
use pallas::crypto::key::ed25519::SecretKey;
use serde::Deserialize;

mod crd;
mod node;

pub use crd::*;
pub use node::*;
use tracing::info;

const DEFAULT_NAMESPACE: &str = "hydra-doom";

fn define_namespace() -> String {
    std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.to_string())
}

#[derive(Clone)]
pub struct ClusterState {
    store: kube::runtime::reflector::Store<HydraDoomNode>,
    watcher_handle: Arc<tokio::task::JoinHandle<()>>,
    pub admin_sk: SecretKey,
}

impl ClusterState {
    pub async fn try_new(admin_key_file: &str) -> anyhow::Result<Self> {
        let admin_key_envelope: KeyEnvelope = serde_json::from_reader(
            File::open(admin_key_file).context("unable to open key file")?,
        )?;

        let admin_sk: SecretKey = admin_key_envelope
            .try_into()
            .context("Failed to get secret key from file")?;

        let namespace = define_namespace();
        info!(namespace, "running inside namespace");

        let client = kube::Client::try_default().await?;
        let nodes: kube::Api<crd::HydraDoomNode> = kube::Api::namespaced(client, &namespace);

        let (store, writer) = kube::runtime::reflector::store();

        // Create the infinite reflector stream
        let rf = kube::runtime::reflector(
            writer,
            kube::runtime::watcher(nodes, kube::runtime::watcher::Config::default()),
        );

        let watcher_handle = tokio::spawn(async move {
            let infinite_watch = rf.applied_objects().for_each(|o| ready(()));
            infinite_watch.await;
        });

        Ok(Self {
            store,
            watcher_handle: Arc::new(watcher_handle),
            admin_sk,
        })
    }

    pub fn get_warm_node(&self) -> anyhow::Result<Arc<HydraDoomNode>> {
        self.store
            .state()
            .iter()
            .filter(|n| n.status.as_ref().is_some_and(|s| s.state == "HeadIsOpen"))
            .next()
            .cloned()
            .ok_or(anyhow::anyhow!("no available warm nodes found"))
    }

    pub fn get_all_nodes(&self) -> Vec<Arc<crd::HydraDoomNode>> {
        self.store.state().iter().cloned().collect()
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<Arc<HydraDoomNode>> {
        let ns = define_namespace();
        self.store
            .get(&ObjectRef::<HydraDoomNode>::new(id).within(&ns))
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ttl_minutes: u64,
}
