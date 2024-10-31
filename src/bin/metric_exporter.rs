use anyhow::Result;
use clap::{arg, Parser};
use hydra_control_plane::model::{
    cluster::{
        metrics::{Metrics, NodeState},
        ConnectionInfo,
    },
    hydra::{
        hydra_message::{HydraData, HydraEventMessage},
        hydra_socket::HydraSocket,
    },
};
use rocket::{get, routes, State};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    host: String,
    #[arg(long)]
    port: u32,
    #[arg(long, action)]
    secure: bool,
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
        .manage(metrics)
        .mount("/", routes![metrics_endpoint])
        .launch()
        .await?;
    Ok(())
}

#[get("/metrics")]
fn metrics_endpoint(metrics: &State<Arc<Metrics>>) -> String {
    metrics.gather()
}

async fn update_connection_state(metrics: Arc<Metrics>, socket: Arc<HydraSocket>) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let current_value = metrics.state.get();
        let is_online = socket.online.load(std::sync::atomic::Ordering::SeqCst);

        if !is_online {
            metrics.set_state(NodeState::Offline);
        } else if current_value == 0 {
            metrics.set_state(NodeState::Online);
        };
    }
}

async fn update(metrics: Arc<Metrics>, mut rx: UnboundedReceiver<HydraData>) {
    loop {
        match rx.recv().await {
            Some(HydraData::Received { message, .. }) => {
                match message {
                    HydraEventMessage::HeadIsOpen(head_is_open) => {
                        info!("head_id {:?}", head_is_open.head_id);
                        metrics.set_state(NodeState::HeadIsOpen);
                    }
                    HydraEventMessage::SnapshotConfirmed(_) => {
                        // Calculate kills, amount of transactions, etc.
                    }
                    HydraEventMessage::CommandFailed(command_failed) => {
                        println!("command failed {:?}", command_failed);
                    }
                    HydraEventMessage::HeadIsInitializing(_) => {
                        info!("node is initializing a head, marking as occupied");
                        metrics.set_state(NodeState::HeadIsInitializing);
                    }
                    HydraEventMessage::InvalidInput(invalid_input) => {
                        println!("Received InvalidInput: {:?}", invalid_input);
                    }
                    HydraEventMessage::Greetings(greetings) => {
                        match greetings.head_status.as_ref() {
                            "Initializing" => metrics.set_state(NodeState::HeadIsInitializing),
                            "Open" => metrics.set_state(NodeState::HeadIsOpen),
                            _ => metrics.set_state(NodeState::Online),
                        };
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
