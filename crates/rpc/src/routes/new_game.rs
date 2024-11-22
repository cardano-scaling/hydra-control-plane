use anyhow::{anyhow, Context};
use pallas::ledger::addresses::Address;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use serde::Serialize;
use tracing::info;

use crate::model::cluster::{ClusterState, NodeClient};

#[derive(Serialize)]
pub struct NewGameResponse {
    game_id: String,
    ip: String,
    player_state: String,
    admin_pkh: String,
}

#[get("/new_game?<address>")]
pub async fn new_game(address: &str, state: &State<ClusterState>) -> Result<Json<NewGameResponse>> {
    info!("Creating a new game for {}", address);

    let pkh = match Address::from_bech32(address).context("invalid address")? {
        Address::Shelley(shelley) => *shelley.payment().as_hash(),
        _ => return Result::Err(anyhow!("unsupported address type").into()),
    };

    let node = state.select_node_for_new_game().context("error getting warm node")?;
    let node_id = node.metadata.name.clone().expect("node without a name");
    info!(id = node_id, "select node for new game");

    let client = NodeClient::new(node, state.admin_sk.clone(), state.remote, state.network)
        .context("error connecting to node")?;

    info!(id = node_id, "connected to node");

    let tx_hash = client
        .new_game(pkh.into())
        .await
        .context("error creating new game")?;

    let ip = client
        .resource
        .status
        .as_ref()
        .map(|status| status.external_url.clone())
        .unwrap_or_default();

    Ok(Json(NewGameResponse {
        game_id: node_id,
        ip,
        player_state: format!("{}#1", hex::encode(tx_hash)),
        admin_pkh: hex::encode(client.tx_builder.admin_pkh),
    }))
}
