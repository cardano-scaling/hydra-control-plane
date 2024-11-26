use std::collections::HashMap;

use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

use crate::model::cluster::ClusterState;

#[derive(Serialize)]
pub struct AddPlayerResponse {
    ip: String,
    player_state: String,
    admin_pkh: String,
}

#[get("/add_player?<address>&<id>")]
pub async fn add_player(
    address: &str,
    id: &str,
    state: &State<ClusterState>,
) -> Result<Json<AddPlayerResponse>, Status> {
    let node = state.get_node_by_id(id).ok_or(Status::NotFound)?;

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

    let url = local_url + "/game/add_player?address=" + address;
    let response = reqwest::get(url).await.map_err(|_| Status::BadGateway)?;

    let body = response
        .json::<HashMap<String, String>>()
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AddPlayerResponse {
        ip: external_url,
        player_state: body
            .get("player_state")
            .ok_or(Status::InternalServerError)?
            .to_owned(),
        admin_pkh: body
            .get("admin_pkh")
            .ok_or(Status::InternalServerError)?
            .to_owned(),
    }))
}
