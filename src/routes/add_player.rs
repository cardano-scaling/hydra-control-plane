use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

use crate::{model::node::Node, MyState};

#[derive(Serialize)]
pub struct AddPlayerResponse {
    ip: String,
    player_state: String,
}

#[get("/add_player?<address>&<id>")]
pub async fn add_player(
    address: &str,
    id: &str,
    state: &State<MyState>,
) -> Result<Json<AddPlayerResponse>, Status> {
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
        .add_player(pkh)
        .await
        .map_err(|_| Status::InternalServerError)?;
    let ip = node.remote_connection.to_authority();

    Ok(Json(AddPlayerResponse {
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
    }))
}
