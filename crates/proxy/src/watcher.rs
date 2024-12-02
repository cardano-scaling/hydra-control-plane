use futures_util::TryStreamExt;
use hydra_control_plane_operator::HydraDoomNode;
use kube::{
    runtime::watcher::{self, Config, Event},
    Api, Client, ResourceExt,
};
use std::sync::Arc;
use tokio::pin;
use tracing::{error, info, instrument};

use crate::State;

#[instrument("auth background service", skip_all)]
pub fn start(state: Arc<State>) {
    tokio::spawn(async move {
        let client = Client::try_default()
            .await
            .expect("failed to create kube client");

        let api = Api::<HydraDoomNode>::all(client.clone());
        let stream = watcher::watcher(api.clone(), Config::default());
        pin!(stream);

        loop {
            let result = stream.try_next().await;
            match result {
                // Stream restart, also run on startup.
                Ok(Some(Event::Init)) => {
                    info!("restarting watcher");
                    state.nodes.write().await.clear();
                }

                // New node created or updated.
                Ok(Some(Event::InitApply(crd))) => {
                    let key = crd.name_any();
                    info!("watcher: Adding node: {}", key);
                    state.nodes.write().await.insert(key, crd);
                }

                Ok(Some(Event::InitDone)) => {
                    info!("restarted watcher.");
                }

                // New node created or updated.
                Ok(Some(Event::Apply(crd))) => {
                    let key = crd.name_any();
                    info!("watcher: Adding node: {}", key);
                    state.nodes.write().await.insert(key, crd);
                }

                // node deleted.
                Ok(Some(Event::Delete(crd))) => {
                    info!(
                        "watcher: Node deleted, removing from state: {}",
                        crd.name_any()
                    );
                    state.nodes.write().await.remove(&crd.name_any());
                }

                // Empty response from stream. Should never happen.
                Ok(None) => {
                    error!("watcher: Empty response from watcher.");
                    continue;
                }
                // Unexpected error when streaming CRDs.
                Err(err) => {
                    error!(error = err.to_string(), "watcher: Failed to update crds.");
                    std::process::exit(1);
                }
            }
        }
    });
}
