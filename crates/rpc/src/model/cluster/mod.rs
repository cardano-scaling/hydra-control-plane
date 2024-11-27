use std::collections::HashMap;
use std::fs::File;
use std::sync::Mutex;
use std::{future::ready, sync::Arc};

use anyhow::Context;
use futures_util::StreamExt as _;
use kube::runtime::{reflector::ObjectRef, WatchStreamExt as _};
use pallas::crypto::key::ed25519::SecretKey;
use pallas::ledger::addresses::Network;
use serde::Deserialize;

mod crd;
mod node;
pub mod shared;

pub use crd::*;
pub use node::*;
use tracing::info;

const DEFAULT_NAMESPACE: &str = "hydra-doom";

fn define_namespace() -> String {
    std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.to_string())
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ClusterState {
    recently_claimed: Arc<Mutex<HashMap<String, bool>>>,
    store: kube::runtime::reflector::Store<HydraDoomNode>,
    watcher_handle: Arc<tokio::task::JoinHandle<()>>,
    pub admin_sk: SecretKey,
    pub remote: bool,
    pub network: Network,
}

impl ClusterState {
    pub async fn try_new(
        admin_key_file: &str,
        remote: bool,
        network: Network,
    ) -> anyhow::Result<Self> {
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

        let claims_ = Arc::new(Mutex::new(HashMap::new()));
        let claims = claims_.clone();

        let watcher_handle = tokio::spawn(async move {
            let infinite_watch = rf.applied_objects().for_each(|node| {
                if let Ok(node) = node {
                    if node
                        .status
                        .as_ref()
                        .is_some_and(|n| n.game_state != "Waiting")
                    {
                        let id = node.metadata.name.as_ref().unwrap();
                        let mut claims = claims.lock().unwrap();
                        if claims.remove(id).is_some() {
                            info!(namespace, "node {} is now available", id);
                        }
                    }
                }
                ready(())
            });
            infinite_watch.await;
        });

        Ok(Self {
            recently_claimed: claims_.clone(),
            store,
            watcher_handle: Arc::new(watcher_handle),
            admin_sk,
            remote,
            network,
        })
    }

    pub fn select_node_for_new_game(&self) -> anyhow::Result<Arc<HydraDoomNode>> {
        let mut claimed = self.recently_claimed.lock().unwrap();
        let node = self
            .store
            .state()
            .iter()
            .filter(|n| {
                let id = n.metadata.name.as_ref().unwrap();
                let recently_claimed = claimed.get(id).unwrap_or(&false);
                info!(
                    "checking node {}, recently claimed: {}, status: {}",
                    id,
                    recently_claimed,
                    n.status
                        .as_ref()
                        .map(|s| s.game_state.as_str())
                        .unwrap_or("unknown")
                );
                if let Some(status) = n.status.as_ref() {
                    !recently_claimed
                        && status.node_state == "HeadIsOpen"
                        && status.game_state == "Waiting"
                } else {
                    false
                }
            })
            .max_by_key(|n| n.metadata.creation_timestamp.clone())
            .cloned()
            .ok_or(anyhow::anyhow!("no available nodes found"))?;
        claimed
            .entry(node.metadata.name.clone().expect("node without a name"))
            .or_insert(true);
        Ok(node)
    }

    pub fn get_all_nodes(&self) -> Vec<Arc<crd::HydraDoomNode>> {
        self.store.state().to_vec()
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
