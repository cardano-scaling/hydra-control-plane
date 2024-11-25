use hydra_control_plane_rpc::model::cluster::{ConnectionInfo, NodeClient};
use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;
use tracing::error;

use crate::LocalState;

#[derive(Serialize)]
pub struct AddPlayerResponse {
    player_state: String,
    admin_pkh: String,
}

#[get("/game/add_player?<address>")]
pub async fn add_player(
    address: &str,
    state: &State<LocalState>,
) -> Result<Json<AddPlayerResponse>, Status> {
    let pkh = match Address::from_bech32(address).map_err(|_| Status::BadRequest)? {
        Address::Shelley(shelley) => Ok(*shelley.payment().as_hash()),
        _ => Err(Status::BadRequest),
    }?;

    let client = NodeClient::new(ConnectionInfo::local(), state.admin_key.clone());

    let tx_hash = client
        .add_player(pkh.into())
        .await
        .inspect_err(|err| error!("error adding player: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AddPlayerResponse {
        player_state: format!("{}#1", hex::encode(tx_hash)),
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
    }))
}
