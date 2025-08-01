use std::path::PathBuf;

use anyhow::{Context, Result};
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
mod routes;

// this is a temporary way to store the script address
pub const SCRIPT_ADDRESS: &str = "addr_test1wrs939u7ve2yqpflwgvf8r5mlh0fmfx6stk9kg00w0kmt5scw3h0h";
pub const SCRIPT_CBOR: &str = "59038e010000323232323232232323232232253330094a229309b2b19299980418020008a99980598051baa00214985854ccc020c0140044c8c94ccc034c03c0084c926330080012533300b3007300c375400226464646464646464646464646464646464646464a66604460480042930b1bad30220013022002375a604000260400046eb4c078004c078008dd6980e000980e0011bad301a001301a002375a603000260300046eb4c058004c058008dd6980a000980a0011bad30120013012002375a6020002601a6ea80045858dd6180680098051baa0021630083754002646464a666010600860126ea801c4c8c8c8c8c8c8c8c8c8c8c8c8c8c94ccc064c06c0084c8c8c8c8c8c926533301b3017301c375400c26464646464646464a66604c60500042930b19299981318128008a999811981018120008a5115333023301f302400114a02c2c6ea8c098004c098008dd6981200098120011bad30220013022002375a6040002603a6ea801858cc06001c8dd68009980b8041180a000a99980c180a180c9baa00913232323232323232323253330253027002132323232498c080018c07c01cc074020c94ccc08cc07c00454ccc098c094dd50050a4c2c2a66604660400022a66604c604a6ea802852616153330233370e90020008a99981318129baa00a14985858c08cdd50048b1bad302500130250023023001302300230210013021002301f001301f002301d001301a37540122c6020014601e0162c603200260320046eb0c05c004c05c008dd6180a800980a80118098009809801180880098088011807800980780119299980698060008a999805180398058008a511533300a3006300b00114a02c2c6ea8c034004c028dd50038b1192999804980280089919299980718080010a4c2c6eb8c038004c02cdd50010a999804980300089919299980718080010a4c2c6eb8c038004c02cdd50010b18049baa00125333007300330083754002264646464a66601c6020004264932999805980398061baa003132323232323253330143016002149858dd6980a000980a0011bad30120013012002375a6020002601a6ea800c5858dd698070009807001180600098049baa00116253330063002300737540022646464646464a66601e60220042930b1bad300f001300f002375a601a002601a0046eb4c02c004c020dd50008b1b8748000dc3a400444646600200200644a66601200229309919801801980600118019805000ab9a5573aaae7955cfaba157441";
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

#[derive(Debug, Deserialize)]
struct NodeConfig {
    #[serde(default = "localhost")]
    local_url: String,
    remote_url: Option<String>,
    #[serde(default = "default_region")]
    region: String,
    port: u32,

    stats_file: Option<String>,

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
