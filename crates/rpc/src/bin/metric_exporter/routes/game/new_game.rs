use anyhow::{anyhow, Context};
use hydra_control_plane_rpc::model::cluster::{shared::NewGameLocalResponse, NodeClient};
use pallas::ledger::addresses::Address;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use tracing::info;

use crate::LocalState;

#[get("/game/elimination")]
pub async fn elimination_game(
    state: &State<LocalState>,
) -> Result<Json<NewGameLocalResponse>> {
    info!("Creating a new elimination game");

    let client = NodeClient::new(state.hydra.clone(), state.admin_key.clone(), state.network);

    let tx_hash = client.new_game(None, 2, 0).await.context("error creating new game")?;
    Ok(Json(NewGameLocalResponse {
        player_state: None,
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
        game_tx_hash: hex::encode(tx_hash),
    }))
}

#[get("/game/new_game?<address>&<player_count>&<bot_count>")]
pub async fn new_game(
    address: &str,
    player_count: u64,
    bot_count: u64,
    state: &State<LocalState>,
) -> Result<Json<NewGameLocalResponse>> {
    info!("Creating a new game for {}", address);

    let pkh = match Address::from_bech32(address).context("invalid address")? {
        Address::Shelley(shelley) => *shelley.payment().as_hash(),
        _ => return Result::Err(anyhow!("unsupported address type").into()),
    };

    let client = NodeClient::new(state.hydra.clone(), state.admin_key.clone(), state.network);

    let tx_hash = client
        .new_game(Some(pkh.into()), player_count, bot_count)
        .await
        .context("error creating new game")?;

    Ok(Json(NewGameLocalResponse {
        player_state: Some(format!("{}#1", hex::encode(tx_hash.clone()))),
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
        game_tx_hash: hex::encode(tx_hash.clone()),
    }))
}
