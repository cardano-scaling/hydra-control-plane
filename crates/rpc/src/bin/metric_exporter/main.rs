use anyhow::{Context, Result};
use clap::{arg, Parser};
use hydra_control_plane_rpc::model::{
    cluster::{ConnectionInfo, KeyEnvelope},
    hydra::{
        hydra_message::{HydraData, HydraEventMessage},
        hydra_socket::HydraSocket,
    },
};
use pallas::{crypto::key::ed25519::SecretKey, ledger::addresses::Network};
use rocket::{get, post, routes, State};
use routes::game::{
    add_player::add_player, cleanup::cleanup, end_game::end_game as node_end_game,
    new_game::{elimination_game, new_game}, start_game::start_game as node_start_game,
};
use std::{env, fs::File, sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{error, info, warn};

mod metrics;
mod routes;
use metrics::{Metrics, NodeState};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    host: String,
    #[arg(long)]
    port: u32,
    #[arg(long, action)]
    secure: bool,
    #[arg(long)]
    admin_key_file: String,
}

pub struct LocalState {
    network: Network,
    hydra: ConnectionInfo,
    admin_key: SecretKey,
    metrics: Arc<Metrics>,
}

#[rocket::main]
async fn main() -> Result<()> {
    let (tx, rx): (UnboundedSender<HydraData>, UnboundedReceiver<HydraData>) =
        mpsc::unbounded_channel();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    let connection_info = ConnectionInfo {
        host: args.host,
        port: args.port,
        secure: args.secure,
    };

    let admin_key_envelope: KeyEnvelope = serde_json::from_reader(
        File::open(args.admin_key_file).context("unable to open key file")?,
    )?;

    let admin_key: SecretKey = admin_key_envelope
        .try_into()
        .context("Failed to get secret key from file")?;

    let network: Network = env::var("NETWORK_ID")
        .map(|network_str| {
            network_str
                .parse::<u8>()
                .inspect_err(|_| error!("Invalid NETWORK_ID value, defaulting to 0"))
                .unwrap_or_default()
        })
        .inspect_err(|_| error!("Missing NETWORK_ID env var, defaulting to zero"))
        .unwrap_or_default()
        .into();

    let socket = Arc::new(HydraSocket::new(
        connection_info.to_websocket_url().as_str(),
        &connection_info.to_authority(),
        &tx,
    ));
    let metrics = Arc::new(Metrics::try_new().expect("Failed to register metrics."));

    // Initialize websocket.
    socket.listen();

    // Check online status.
    tokio::spawn(update_connection_state(metrics.clone(), socket.clone()));
    // Listen and update metrics.
    tokio::spawn(update(metrics.clone(), rx));

    let _ = rocket::build()
        .manage(LocalState {
            admin_key,
            hydra: connection_info,
            metrics,
            network,
        })
        .mount(
            "/",
            routes![
                metrics_endpoint,
                start_server,
                start_game,
                end_game,
                player_joined,
                player_left,
                player_killed,
                player_suicided,
                new_game,
                elimination_game,
                add_player,
                node_start_game,
                node_end_game,
                cleanup,
            ],
        )
        .launch()
        .await?;

    Ok(())
}

#[get("/metrics")]
fn metrics_endpoint(state: &State<LocalState>) -> String {
    state.metrics.gather()
}

#[post("/start_server")]
fn start_server(state: &State<LocalState>) {
    state.metrics.start_server();
}

#[post("/start_game")]
fn start_game(state: &State<LocalState>) {
    state.metrics.start_game();
}

#[post("/end_game")]
fn end_game(state: &State<LocalState>) {
    state.metrics.end_game();
}

#[post("/player_joined")]
fn player_joined(state: &State<LocalState>) {
    state.metrics.player_joined();
}

#[post("/player_left")]
fn player_left(state: &State<LocalState>) {
    state.metrics.player_left();
}

#[post("/player_killed")]
fn player_killed(state: &State<LocalState>) {
    state.metrics.player_killed();
}

#[post("/player_suicided")]
fn player_suicided(state: &State<LocalState>) {
    state.metrics.player_suicided();
}

async fn update_connection_state(metrics: Arc<Metrics>, socket: Arc<HydraSocket>) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let current_value = metrics.node_state.get();
        let is_online = socket.online.load(std::sync::atomic::Ordering::SeqCst);

        if !is_online {
            metrics.set_node_state(NodeState::Offline);
        } else if current_value == 0 {
            metrics.set_node_state(NodeState::Online);
        };
    }
}

async fn update(metrics: Arc<Metrics>, mut rx: UnboundedReceiver<HydraData>) {
    loop {
        match rx.recv().await {
            Some(HydraData::Received { message, .. }) => match message {
                HydraEventMessage::HeadIsOpen(head_is_open) => {
                    info!("head_id {:?}", head_is_open.head_id);
                    metrics.set_node_state(metrics::NodeState::HeadIsOpen);
                }
                HydraEventMessage::CommandFailed(command_failed) => {
                    println!("command failed {:?}", command_failed);
                }
                HydraEventMessage::HeadIsInitializing(_) => {
                    info!("node is initializing a head, marking as occupied");
                    metrics.set_node_state(NodeState::HeadIsInitializing);
                }
                HydraEventMessage::InvalidInput(invalid_input) => {
                    println!("Received InvalidInput: {:?}", invalid_input);
                }
                HydraEventMessage::Greetings(greetings) => {
                    match greetings.head_status.as_ref() {
                        "Initializing" => metrics.set_node_state(NodeState::HeadIsInitializing),
                        "Open" => metrics.set_node_state(NodeState::HeadIsOpen),
                        _ => metrics.set_node_state(NodeState::Online),
                    };
                }
                HydraEventMessage::TxValid(valid) => {
                    metrics.new_transaction(valid.transaction.cbor.len() as u64);
                }
                _ => {}
            },
            Some(HydraData::Send(_)) => {}
            None => {
                warn!("mpsc disconnected");
                break;
            }
        }
    }
}
