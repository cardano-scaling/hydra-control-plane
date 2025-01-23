use anyhow::anyhow;
use hydra_control_plane_rpc::model::cluster::NodeClient;
use rocket::{post, State};
use rocket_errors::anyhow::Result;

use crate::LocalState;

#[post("/game/cleanup")]
pub async fn cleanup(state: &State<LocalState>) -> Result<()> {
    let client = NodeClient::new(state.hydra.clone(), state.admin_key.clone(), state.network);
    let series_utxo_ref = {
        state
            .series_utxo
            .read()
            .map_err(|_| anyhow!("failed to read state"))?
            .clone()
            .ok_or(anyhow!("no series utxo"))?
    };

    let played_games = {
        *state
            .game_count
            .read()
            .map_err(|_| anyhow!("failed to read state"))?
    };

    client
        .cleanup_game(series_utxo_ref, played_games)
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}
