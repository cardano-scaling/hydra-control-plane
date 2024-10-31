use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

use crate::model::cluster::ClusterState;

#[derive(Serialize)]
pub struct AddPlayerResponse {
    ip: String,
    player_state: String,
}

#[get("/add_player?<address>&<id>")]
pub async fn add_player(
    address: &str,
    id: &str,
    state: &State<ClusterState>,
) -> Result<Json<AddPlayerResponse>, Status> {
    let node = state
        .get_node_by_id(id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    // if we don't find the node for the given game id, return a 404
    let node = node.ok_or(Status::NotFound)?;

    let pkh = match Address::from_bech32(address).map_err(|_| Status::BadRequest)? {
        Address::Shelley(shelley) => Ok(shelley.payment().as_hash().clone()),
        _ => Err(Status::BadRequest),
    }?;

    let tx_hash = node
        .add_player(pkh)
        .await
        .map_err(|_| Status::InternalServerError)?;
    let ip = node.remote_connection.to_http_url();

    Ok(Json(AddPlayerResponse {
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
    }))
}
