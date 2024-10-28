use itertools::Itertools;
use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

use crate::{model::node::Node, MyState};

#[derive(Serialize)]
pub struct NewGameResponse {
    ip: String,
    player_state: String,
}

#[get("/new_game?<address>")]
pub async fn new_game(
    address: &str,
    state: &State<MyState>,
) -> Result<Json<NewGameResponse>, Status> {
    let state_guard = state.state.state.write().await;
    // we're just gonna grab the first node for now.
    // In the future, we will be querying K8s
    let node: &Node = state_guard
        .nodes
        .get(0)
        .ok_or(Status::InternalServerError)?;

    let pkh = match Address::from_bech32(address).map_err(|_| Status::BadRequest)? {
        Address::Shelley(shelley) => Ok(shelley.payment().as_hash().clone()),
        _ => Err(Status::BadRequest),
    }?;

    let tx_hash = node
        .new_game(pkh)
        .await
        .map_err(|_| Status::InternalServerError)?;
    let ip = node.remote_connection.to_http_url();

    Ok(Json(NewGameResponse {
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
    }))
}
