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

    // let node2 = Node::try_new("ws://3.15.33.186:4001", &tx)
    //     .await
    //     .expect("failed to connect");

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
                HydraData::Received { message, uri } => {
                    let mut state_guard = state.state.write().await;
                    let nodes = &mut state_guard.nodes;
                    let node = nodes.iter_mut().find(|n| n.uri == uri);
                    if let None = node {
                        println!("Node not found: ${:?}", uri);
                        continue;
                    }
                    let node = node.unwrap();
                    match message {
                        HydraEventMessage::HeadIsOpen(head_is_open) => {
                            if let None = node.head_id {
                                println!(
                                    "updating node {:?} with head_id {:?}",
                                    node.uri,
                                    head_is_open.head_id()
                                );
                                node.head_id = Some(head_is_open.head_id().to_string());
                            }
                        }
                        HydraEventMessage::SnapshotConfirmed(snapshot_confirmed) => node
                            .stats
                            .calculate_stats(snapshot_confirmed.confirmed_transactions),
                        HydraEventMessage::TxValid(tx) => {
                            node.stats.add_transaction(tx.tx_id.clone(), tx.into());
                        }
                        _ => println!("Unhandled message: {:?}", message),
                    }
                }
                HydraData::Sent(_) => {}
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                println!("mpsc disconnected");
                break;
            }
        }
    }
}
