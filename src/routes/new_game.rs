use ::anyhow::{anyhow, Context};
use pallas::ledger::addresses::Address;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::{self, AnyhowError, Result};
use serde::Serialize;
use tracing::info;

use crate::model::cluster::{ClusterState, NodeClient};

#[derive(Serialize)]
pub struct NewGameResponse {
    game_id: String,
    ip: String,
    player_state: String,
}

#[get("/new_game?<address>")]
pub async fn new_game(address: &str, state: &State<ClusterState>) -> Result<Json<NewGameResponse>> {
    info!("Creating a new game for {}", address);

    let pkh = match Address::from_bech32(address).context("invalid address")? {
        Address::Shelley(shelley) => shelley.payment().as_hash().clone(),
        _ => return Result::Err(anyhow!("unsupported address type").into()),
    };

    let node = state.get_warm_node().context("error getting warm node")?;

    info!(id = &node.metadata.name, "select node for new game");

    let client =
        NodeClient::new(node, state.admin_sk.clone(), true).context("error connecting to node")?;

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
        node.metadata.name,
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
    }))
}
