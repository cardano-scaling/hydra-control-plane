use itertools::Itertools;
use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

use crate::model::cluster::{ClusterState, Node};

#[derive(Serialize)]
pub struct NewGameResponse {
    ip: String,
    player_state: String,
}

#[get("/new_game?<address>")]
pub async fn new_game(
    address: &str,
    state: &State<ClusterState>,
) -> Result<Json<NewGameResponse>, Status> {
    let node = state
        .get_warm_node()
        .await
        .map_err(|_| Status::InternalServerError)?;

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
