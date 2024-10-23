use std::path::PathBuf;

use anyhow::{Context, Result};
use hydra_control_plane::NodeConfig;
use model::{
    hydra::{
        hydra_message::{HydraData, HydraEventMessage},
        state::HydraNodesState,
    },
    node::Node,
};
use rocket::{http::Method, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use routes::{global::global, head::head, heads::heads, new_game::new_game};
use serde::Deserialize;
use tokio::{
    spawn,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tracing::{info, warn};

mod model;
mod providers;
mod routes;

pub struct MyState {
    state: HydraNodesState,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ttl_minutes: u64,
    #[serde(default = "default_hosts")]
    hosts: Vec<HostConfig>,
    #[serde(default = "default_nodes")]
    nodes: Vec<NodeConfig>,
}

fn default_nodes() -> Vec<NodeConfig> {
    vec![]
}
fn default_hosts() -> Vec<HostConfig> {
    vec![]
}

#[derive(Debug, Deserialize)]
struct HostConfig {
    #[serde(default = "localhost")]
    local_url: String,
    remote_url: Option<String>,
    stats_file_prefix: Option<String>,
    region: String,
    #[serde(default = "default_start_port")]
    start_port: u32,
    #[serde(default = "default_start_port")]
    end_port: u32,

    max_players: usize,
    admin_key_file: PathBuf,
    persisted: bool,
    reserved: bool,
}

fn default_start_port() -> u32 {
    4001
}

fn default_region() -> String {
    "us-east-2".to_string()
}

fn localhost() -> String {
    "ws://127.0.0.1".to_string()
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().context("invalid config")?;

    let (tx, rx): (UnboundedSender<HydraData>, UnboundedReceiver<HydraData>) =
        mpsc::unbounded_channel();

    let mut nodes = vec![];
    for node in &config.nodes {
        let node = Node::try_new(node, &tx)
            .await
            .context("failed to construct new node")?;
        nodes.push(node);
    }
    for host in &config.hosts {
        for port in host.start_port..=host.end_port {
            let config = NodeConfig {
                local_url: host.local_url.clone(),
                remote_url: host.remote_url.clone(),
                region: host.region.clone(),
                port,
                stats_file: host
                    .stats_file_prefix
                    .as_ref()
                    .map(|prefix| format!("{prefix}-{port}")),
                admin_key_file: host.admin_key_file.clone(),
                max_players: host.max_players,
                persisted: host.persisted,
                reserved: host.reserved,
            };
            let node = Node::try_new(&config, &tx)
                .await
                .context("failed to construct new node")?;
            nodes.push(node);
        }
    }

    let hydra_state = HydraNodesState::from_nodes(nodes);

    let hydra_state_clone = hydra_state.clone();
    spawn(async move {
        update(hydra_state_clone, rx).await;
    });

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _rocket = rocket::build()
        .manage(MyState { state: hydra_state })
        .mount("/", routes![new_game, heads, head, global])
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;

    Ok(())
}

async fn update(state: HydraNodesState, mut rx: UnboundedReceiver<HydraData>) {
    loop {
        match rx.recv().await {
            Some(HydraData::Received { message, authority }) => {
                let mut state_guard = state.state.write().await;
                let nodes = &mut state_guard.nodes;
                let node = nodes
                    .iter_mut()
                    .find(|n| n.local_connection.to_authority() == authority);
                if node.is_none() {
                    warn!("Node not found: {}", authority);
                    continue;
                }
                let node = node.unwrap();
                match message {
                    HydraEventMessage::HeadIsOpen(head_is_open) if node.head_id.is_none() => {
                        info!(
                            "updating node {:?} with head_id {:?}",
                            node.local_connection.to_authority(),
                            head_is_open.head_id
                        );
                        node.head_id = Some(head_is_open.head_id.to_string());
                    }
                    HydraEventMessage::SnapshotConfirmed(snapshot_confirmed) => {
                        node.stats.calculate_stats(
                            snapshot_confirmed.confirmed_transactions,
                            node.stats_file.clone(),
                        );
                    }

                    HydraEventMessage::TxValid(tx) => match node.add_transaction(tx) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("failed to add transaction {:?}", e);
                        }
                    },
                    HydraEventMessage::CommandFailed(command_failed) => {
                        println!("command failed {:?}", command_failed);
                    }
                    HydraEventMessage::HeadIsInitializing(_) => {
                        info!(
                            "node {:?} is initializing a head, marking as occupied",
                            node.local_connection.to_authority()
                        );
                        node.occupied = true;
                    }
                    _ => {}
                }
            }
            Some(HydraData::Send(_)) => {}
            None => {
                warn!("mpsc disconnected");
                break;
            }
        }
    }
}
