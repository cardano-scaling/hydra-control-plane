use crate::routes::get_node::get_node;
use std::sync::Mutex;

use model::node::Node;
use serde::Deserialize;

#[macro_use]
extern crate rocket;

mod model;
mod routes;

struct MyState {
    nodes: Mutex<Vec<Node>>,
    config: Config,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Config {
    ttl_minutes: u64,
    max_players: u64,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let config = figment.extract::<Config>().expect("invalid config");
    let node = Node::try_new("ws://127.0.0.1:4001")
        .await
        .expect("failed to connect");
    let nodes = vec![node];

    let _rocket = rocket::build()
        .manage(MyState {
            nodes: Mutex::new(nodes),
            config,
        })
        .mount("/", routes![get_node])
        .launch()
        .await?;

    Ok(())
}
