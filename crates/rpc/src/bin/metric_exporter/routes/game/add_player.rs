use hydra_control_plane_rpc::model::cluster::{shared::AddPlayerLocalResponse, NodeClient};
use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use tracing::error;

use crate::LocalState;

#[get("/game/add_player?<address>")]
pub async fn add_player(
    address: &str,
    state: &State<LocalState>,
) -> Result<Json<AddPlayerLocalResponse>, Status> {
    let pkh = match Address::from_bech32(address).map_err(|_| Status::BadRequest)? {
        Address::Shelley(shelley) => Ok(*shelley.payment().as_hash()),
        _ => Err(Status::BadRequest),
    }?;

    let client = NodeClient::new(state.hydra.clone(), state.admin_key.clone(), state.network);

    let tx_hash = client
        .add_player(pkh.into())
        .await
        .inspect_err(|err| error!("error adding player: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AddPlayerLocalResponse {
        player_state: format!("{}#1", hex::encode(tx_hash)),
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
    }))
}
