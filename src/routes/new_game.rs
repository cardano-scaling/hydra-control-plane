use ::anyhow::{Context, anyhow};
use hydra_control_plane::TEMP_ADMIN_KEY;
use pallas::ledger::addresses::Address;
use rocket::{get, serde::json::Json, State};
use serde::Serialize;
use tracing::info;
use rocket_errors::anyhow::{self, Result, AnyhowError};

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
) -> Result<Json<NewGameResponse>> {
    info!("Creating a new game for {}", address);

    let pkh = match Address::from_bech32(address).context("invalid address")? {
        Address::Shelley(shelley) => shelley.payment().as_hash().clone(),
        _ => return Result::Err(anyhow!("unsupported address type").into()),
    };

    let node = state
        .get_warm_node()
        .context("error getting warm node")?;

    info!(id = &node.metadata.name, "select node for new game");

    let client = NodeClient::new(node, TEMP_ADMIN_KEY.clone(), false)
        .context("error connecting to node")?;

    info!(id = &client.resource.metadata.name, "connected to node");

    let tx_hash = client
        .new_game(pkh)
        .await
        .context("error creating new game")?;

    let ip = client
        .resource
        .status
        .as_ref()
        .map(|status| status.external_url.clone())
        .unwrap_or_default();

    Ok(Json(NewGameResponse {
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
    }))
}
