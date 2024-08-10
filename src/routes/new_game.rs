use crate::{
    model::{node::Node, player::Player},
    MyState,
};
use itertools::Itertools;
use pallas::ledger::addresses::Address;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use serde::Serialize;

#[derive(Serialize)]
pub struct NewGameResponse {
    ip: String,
    script_ref: Option<String>,
    admin_pkh: String,
    player_utxo: String,
    player_utxo_datum_hex: String,
}

#[get("/new_game?<address>")]
pub async fn new_game(
    address: &str,
    state: &State<MyState>,
) -> Result<Json<NewGameResponse>, Status> {
    let mut state_guard = state.state.state.write().await;
    let node: &mut Node = state_guard
        .nodes
        .iter_mut()
        .sorted_by_key(|n| n.players.len())
        .next() // Get the first with the fewest players
        .ok_or_else(|| {
            warn!("No nodes available");
            Status::ServiceUnavailable
        })?;

    let addr = Address::from_bech32(address).map_err(|_| Status::BadRequest)?;

    let player = Player::new(addr).map_err(|_| Status::BadRequest)?;
    let (player_utxo, player_utxo_datum_hex) = node.add_player(player).await.map_err(|e| {
        warn!("failed to add player {:?}", e);
        Status::InternalServerError
    })?;

    Ok(Json(NewGameResponse {
        ip: node.remote_connection.to_authority(),
        script_ref: node.tx_builder.script_ref.clone().map(|s| s.to_string()),
        admin_pkh: node.tx_builder.admin_pkh.to_string(),
        player_utxo,
        player_utxo_datum_hex,
    }))
}
