use hydra_control_plane::TEMP_ADMIN_KEY;
use itertools::Itertools;
use pallas::{crypto::key::ed25519::SecretKey, ledger::addresses::Address};
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;
use tracing::{error, info};

use crate::model::cluster::{ClusterState, NodeClient};

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
    let pkh = match Address::from_bech32(address).map_err(|_| Status::BadRequest)? {
        Address::Shelley(shelley) => Ok(shelley.payment().as_hash().clone()),
        _ => Err(Status::BadRequest),
    }?;

    let node = state
        .get_warm_node()
        .inspect_err(|err| error!("error getting warm node: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    info!(id = &node.metadata.name, "select node for new game");

    let client = NodeClient::new(node, TEMP_ADMIN_KEY.clone())
        .inspect_err(|err| error!("error connecting to node: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    info!(id = &client.resource.metadata.name, "connected to node");

    let tx_hash = client
        .new_game(pkh)
        .await
        .inspect_err(|err| error!("error creating new game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    let ip = client.remote_connection.to_http_url();

    Ok(Json(NewGameResponse {
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
    }))
}
