use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;
use tracing::error;

use crate::model::cluster::{ClusterState, NodeClient};

#[derive(Serialize)]
pub struct AddPlayerResponse {
    ip: String,
    player_state: String,
    admin_pkh: String,
}

#[get("/add_player?<address>&<id>")]
pub async fn add_player(
    address: &str,
    id: &str,
    state: &State<ClusterState>,
) -> Result<Json<AddPlayerResponse>, Status> {
    let pkh = match Address::from_bech32(address).map_err(|_| Status::BadRequest)? {
        Address::Shelley(shelley) => Ok(*shelley.payment().as_hash()),
        _ => Err(Status::BadRequest),
    }?;

    let node = state.get_node_by_id(id).ok_or(Status::NotFound)?;

    let client = NodeClient::new(node, state.admin_sk.clone(), state.remote)
        .inspect_err(|err| error!("error connecting to node: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    let tx_hash = client
        .add_player(pkh.into())
        .await
        .inspect_err(|err| error!("error adding player: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    let ip = client
        .resource
        .status
        .as_ref()
        .map(|status| status.external_url.clone())
        .unwrap_or_default();

    Ok(Json(AddPlayerResponse {
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
    }))
}
