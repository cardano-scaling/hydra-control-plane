use routes::global::global;
use routes::head::head;
use routes::heads::heads;
use routes::new_game::new_game;
use tokio::{
    spawn,
    sync::mpsc::{self, error::TryRecvError, UnboundedReceiver, UnboundedSender},
};

use model::{
    hydra::{
        hydra_message::{HydraData, HydraEventMessage},
        state::HydraNodesState,
    },
    node::Node,
};
use serde::Deserialize;

#[macro_use]
extern crate rocket;

mod model;
mod routes;

// this is a temporary way to store the script address
pub const SCRIPT_ADDRESS: &str = "addr_test1wp096khk46y6mxmnl0pqe446kdlzswsjpyd67ju6gs9sldqjkl4wx";

struct MyState {
    state: HydraNodesState,
    config: Config,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Config {
    ttl_minutes: u64,
    max_players: u64,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let (tx, rx): (UnboundedSender<HydraData>, UnboundedReceiver<HydraData>) =
        mpsc::unbounded_channel();

    let node = Node::try_new("ws://127.0.0.1:4001", &tx, true)
        .await
        .expect("failed to connect");

    // let node2 = Node::try_new("ws://3.15.33.186:4001", &tx, true)
    //     .await
    //     .expect("failed to connect");

    // Fetching utxos requires deserializing them, but for some reaosn whne I print them out locally, it hangs after printing
    // println!("{:?}", utxos);

    // println!("Done fetching utxos...");

    let nodes = vec![node];
    let hydra_state = HydraNodesState::from_nodes(nodes);

    let hydra_state_clone = hydra_state.clone();
    spawn(async move {
        update(hydra_state_clone, rx).await;
    });

    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().expect("invalid config");

    let _rocket = rocket::build()
        .manage(MyState {
            state: hydra_state,
            config,
        })
        .mount("/", routes![new_game, heads, head, global])
        .launch()
        .await?;

    Ok(())
}

async fn update(state: HydraNodesState, mut rx: UnboundedReceiver<HydraData>) {
    loop {
        match rx.try_recv() {
            Ok(data) => match data {
                HydraData::Received { message, authority } => {
                    let mut state_guard = state.state.write().await;
                    let nodes = &mut state_guard.nodes;
                    let node = nodes
                        .iter_mut()
                        .find(|n| n.connection_info.to_authority() == authority);
                    if let None = node {
                        println!("Node not found: ${:?}", authority);
                        continue;
                    }
                    let node = node.unwrap();
                    match message {
                        HydraEventMessage::HeadIsOpen(head_is_open) => {
                            if let None = node.head_id {
                                println!(
                                    "updating node {:?} with head_id {:?}",
                                    node.connection_info.to_authority(),
                                    head_is_open.head_id()
                                );
                                node.head_id = Some(head_is_open.head_id().to_string());
                            }
                        }
                        HydraEventMessage::SnapshotConfirmed(snapshot_confirmed) => node
                            .stats
                            .calculate_stats(snapshot_confirmed.confirmed_transactions),
                        HydraEventMessage::TxValid(tx) => match node.add_transaction(tx) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("failed to add transaction {:?}", e);
                            }
                        },
                        _ => println!("Unhandled message: {:?}", message),
                    }
                }
                HydraData::Send(_) => {}
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                println!("mpsc disconnected");
                break;
            }
        }
    }
}
