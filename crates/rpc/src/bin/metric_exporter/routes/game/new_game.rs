use anyhow::{anyhow, Context};
use hydra_control_plane_rpc::model::cluster::{ConnectionInfo, NodeClient};
use pallas::ledger::addresses::Address;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use serde::Serialize;
use tracing::info;

use crate::LocalState;

#[derive(Serialize)]
pub struct NewGameResponse {
    player_state: String,
    admin_pkh: String,
}

#[get("/game/new_game?<address>")]
pub async fn new_game(address: &str, state: &State<LocalState>) -> Result<Json<NewGameResponse>> {
    info!("Creating a new game for {}", address);

    let pkh = match Address::from_bech32(address).context("invalid address")? {
        Address::Shelley(shelley) => *shelley.payment().as_hash(),
        _ => return Result::Err(anyhow!("unsupported address type").into()),
    };

    let client = NodeClient::new(
        ConnectionInfo::local(),
        state.admin_key.clone(),
        state.network,
    );

    let tx_hash = client
        .new_game(pkh.into())
        .await
        .context("error creating new game")?;

    Ok(Json(NewGameResponse {
        player_state: format!("{}#1", hex::encode(tx_hash)),
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
    }))
}
