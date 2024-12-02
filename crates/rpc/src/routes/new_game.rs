use std::sync::Arc;

use anyhow::{anyhow, Context};
use kube::core::response;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use serde::Serialize;
use tracing::info;

use crate::model::cluster::{shared::NewGameLocalResponse, ClusterState, HydraDoomNode};

#[derive(Serialize)]
pub struct NewGameResponse {
    game_id: String,
    ip: String,
    player_state: String,
    admin_pkh: String,
}

pub async fn do_new_game(
    node: Arc<HydraDoomNode>,
    address: &str,
    player_count: Option<u64>,
    bot_count: Option<u64>,
) -> Result<Json<NewGameResponse>> {
    let node_id = node.metadata.name.clone().expect("node without a name");
    info!(id = node_id, "select node for new game");

    let (external_url, local_url): (String, String) = node
        .status
        .as_ref()
        .map(|status| {
            (
                status.external_url.clone(),
                status
                    .local_url
                    .clone()
                    .replace("ws://", "http://")
                    .replace("4001", "8000"),
            )
        })
        .unwrap_or_default();

    let url = format!(
        "{local_url}/game/new_game?address={address}&player_count={}&bot_count={}",
        player_count.unwrap_or(1),
        bot_count.unwrap_or(2)
    );

    let response = reqwest::get(url)
        .await
        .context("failed to hit new_game metrics server endpoint")?;

    let body = response
        .json::<NewGameLocalResponse>()
        .await
        .context("http error")?;

    Ok(Json(NewGameResponse {
        game_id: node_id,
        ip: external_url,
        player_state: body.player_state,
        admin_pkh: body.admin_pkh,
    }))
}

#[get("/new_game?<address>&<player_count>&<bot_count>")]
pub async fn new_game(
    address: &str,
    player_count: Option<u64>,
    bot_count: Option<u64>,
    state: &State<ClusterState>,
) -> Result<Json<NewGameResponse>> {
    info!("Creating a new game for {}", address);
    if player_count.is_some_and(|c| c > 4) {
        return Result::Err(anyhow!("Can request a maximum of 4 players").into());
    }

    if bot_count.is_some_and(|c| c > 4) {
        return Result::Err(anyhow!("Can request a maximum of 4 bots").into());
    }

    if player_count.is_some_and(|c| bot_count.is_some_and(|b| c + b > 4)) {
        return Result::Err(anyhow!("cannot have more than 4 players and bots").into());
    }
    let node = state
        .select_node_for_new_game()
        .context("error getting warm node")?;
    let node_id = node.metadata.name.clone().context("node without a name")?;
    do_new_game(node.clone(), address, player_count, bot_count).await.or_else(|e| {
        state.release_node(node_id.as_str());
        Err(e)
    })
}
