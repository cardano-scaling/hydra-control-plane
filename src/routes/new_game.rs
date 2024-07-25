use crate::{
    model::{node::Node, player::Player},
    MyState,
};
use pallas::ledger::addresses::Address;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use serde::Serialize;

#[derive(Serialize)]
pub struct NewGameResponse {
    ip: String,
    script_ref: Option<String>,
}

#[get("/new_game?<address>")]
pub async fn new_game(
    address: &str,
    state: &State<MyState>,
) -> Result<Json<NewGameResponse>, Status> {
    let max_players: usize = state.config.max_players.try_into().unwrap_or_else(|_| 1);
    let mut state_guard = state.state.state.write().await;
    let node: &mut Node = state_guard
        .nodes
        .iter_mut()
        .find(|n| n.players.len() < max_players)
        .ok_or(Status::ServiceUnavailable)?;

    let addr = Address::from_bech32(address).map_err(|_| Status::BadRequest)?;

    let player = Player::new(addr).map_err(|_| Status::BadRequest)?;
    let _ = node.add_player(player).await.map_err(|e| {
        warn!("failed to add player {:?}", e);
        Status::InternalServerError
    })?;

    Ok(Json(NewGameResponse {
        ip: node.remote_connection.to_authority(),
        script_ref: node.tx_builder.script_ref.clone().map(|s| s.to_string()),
    }))
}
