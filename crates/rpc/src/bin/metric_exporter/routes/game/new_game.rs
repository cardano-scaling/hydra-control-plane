use anyhow::{anyhow, Context};
use hydra_control_plane_rpc::model::cluster::{shared::NewGameLocalResponse, NodeClient};
use pallas::ledger::addresses::Address;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use tracing::info;

use crate::LocalState;

#[get("/game/new_game?<address>")]
pub async fn new_game(
    address: &str,
    state: &State<LocalState>,
) -> Result<Json<NewGameLocalResponse>> {
    let pkh = match Address::from_bech32(address).context("invalid address")? {
        Address::Shelley(shelley) => *shelley.payment().as_hash(),
        _ => return Result::Err(anyhow!("unsupported address type").into()),
    };

    let client = NodeClient::new(state.hydra.clone(), state.admin_key.clone(), state.network);

    let series_exists = {
        state
            .series_utxo
            .read()
            .map_err(|_| anyhow!("Failed to read state"))?
            .is_some()
    };

    if !series_exists {
        return Result::Err(anyhow!("Series does not exist yet").into());
    }

    let should_create_new_game = {
        let is_active = state
            .active_game
            .read()
            .map_err(|_| anyhow!("Failed to read state"))?;

        !*is_active
    };

    let current_game_count = {
        *state
            .game_count
            .read()
            .map_err(|_| anyhow!("Failed to read state"))?
    };

    let tx_hash = if should_create_new_game {
        info!("Creating a new game for {}", address);
        let tx_hash = client
            .new_game(Some(pkh.into()))
            .await
            .context("error creating new game")?;

        state
            .active_game
            .write()
            .map_err(|_| anyhow!("Failed to write state"))?
            .clone_from(&true);

        state
            .game_count
            .write()
            .map_err(|_| anyhow!("failed to write state"))?
            .clone_from(&(current_game_count + 1));

        tx_hash
    } else {
        info!("Joining an existing game for {}", address);
        client
            .add_player(pkh.into())
            .await
            .context("error adding player to game")?
    };

    Ok(Json(NewGameLocalResponse {
        player_state: Some(format!("{}#1", hex::encode(tx_hash.clone()))),
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
    }))
}
