use std::collections::HashMap;

use anyhow::Context;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use serde::Serialize;
use tracing::info;

use crate::model::cluster::ClusterState;

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

    let node = state
        .select_node_for_new_game()
        .context("error getting warm node")?;
    let node_id = node.metadata.name.clone().expect("node without a name");
    info!(id = node_id, "select node for new game");

    let (external_url, local_url): (String, String) = node
        .status
        .as_ref()
        .map(|status| {
            (
                status.external_url.clone(),
                status.local_url.clone().replace("ws://", "http://"),
            )
        })
        .unwrap_or_default();

    let url = local_url + "/game/new_game?address=" + address;
    let response = reqwest::get(url)
        .await
        .context("failed to hit new_game metrics server endpoint")?;
    let body = response
        .json::<HashMap<String, String>>()
        .await
        .context("http error")?;

    Ok(Json(NewGameResponse {
        game_id: node_id,
        ip: external_url,
        player_state: body
            .get("player_state")
            .context("missing player_state in response")?
            .to_owned(),
        admin_pkh: body
            .get("admin_pkh")
            .context("missing admin_pkh in response")?
            .to_owned(),
    }))
}
