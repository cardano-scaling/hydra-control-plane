use std::sync::Arc;

use crate::routes::get_node::get_node;
use tokio::{
    spawn,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use model::{hydra::state::HydraNodesState, node::Node};
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
    let (tx, rx): (UnboundedSender<String>, UnboundedReceiver<String>) = mpsc::unbounded_channel();

    let node = Node::try_new("ws://127.0.0.1:4001", &tx)
        .await
        .expect("failed to connect");
    let nodes = vec![node];
    let hydra_state = HydraNodesState::from_nodes(nodes);

    let hydra_state_clone = hydra_state.clone();
    spawn(async move {
        update(hydra_state_clone, rx);
    });

    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().expect("invalid config");

    let _rocket = rocket::build()
        .manage(MyState {
            state: hydra_state,
            config,
        })
        .mount("/", routes![get_node])
        .launch()
        .await?;

    Ok(())
}

fn update(state: HydraNodesState, reader: UnboundedReceiver<String>) {
    //
    // ... handle updating
}
