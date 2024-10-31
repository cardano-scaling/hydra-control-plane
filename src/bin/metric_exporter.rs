use hydra_control_plane::model::hydra::{
    hydra_message::{HydraData, HydraEventMessage},
    hydra_socket::HydraSocket,
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{info, warn};

async fn update(metrics: Metrics, mut rx: UnboundedReceiver<HydraData>) {
    loop {
        match rx.recv().await {
            Some(HydraData::Received { message, .. }) => {
                match message {
                    HydraEventMessage::HeadIsOpen(head_is_open) => {
                        info!("head_id {:?}", head_is_open.head_id);
                    }
                    HydraEventMessage::SnapshotConfirmed(snapshot_confirmed) => {
                        // node.stats.calculate_stats(
                        //     snapshot_confirmed.confirmed_transactions,
                        //     node.stats_file.clone(),
                        // );
                    }

                    // HydraEventMessage::TxValid(tx) => match node.add_transaction(tx) {
                    //     Ok(_) => {}
                    //     Err(e) => {
                    //         warn!("failed to add transaction {:?}", e);
                    //     }
                    // },
                    HydraEventMessage::CommandFailed(command_failed) => {
                        println!("command failed {:?}", command_failed);
                    }
                    HydraEventMessage::HeadIsInitializing(_) => {
                        info!("node is initializing a head, marking as occupied");
                        // TODO: mark as occupied
                    }
                    HydraEventMessage::InvalidInput(invalid_input) => {
                        println!("Received InvalidInput: {:?}", invalid_input);
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

// TODO: replace with prometheus exporter crate registry
pub struct Metrics;

#[tokio::main]
async fn main() {
    let (tx, rx): (UnboundedSender<HydraData>, UnboundedReceiver<HydraData>) =
        mpsc::unbounded_channel();

    let socket = HydraSocket::new("ws://127.0.0.1:3000?history=no", "127.0.0.1:3000", &tx);

    let metrics = Metrics;

    tokio::spawn(async move {
        update(metrics, rx).await;
    });
}
