use std::sync::atomic::Ordering;

use itertools::Itertools;
use pallas::ledger::addresses::Address;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;
use tracing::warn;

use crate::{
    model::{node::Node, player::Player},
    MyState,
};

#[derive(Serialize)]
pub struct NewGameResponse {
    ip: String,
    script_ref: String,
    admin_pkh: String,
    player_utxo: String,
    player_utxo_datum_hex: String,
}

#[get("/new_game")]
pub async fn new_game(
    // address: &str,
    // region: Option<&str>,
    // reserved: bool,
    state: &State<MyState>,
) -> Result<Json<NewGameResponse>, Status> {
    let mut state_guard = state.state.state.write().await;
    let node: &mut Node = state_guard
        .nodes
        .iter_mut()
        // Only direct games to online games
        .filter(|n| n.socket.online.load(Ordering::SeqCst) && !n.occupied)
        .next() // Get the first with the fewest players
        .ok_or_else(|| {
            warn!("No nodes available");
            Status::ServiceUnavailable
        })?;
    Err(Status::NotImplemented)

    // let node: &mut Node = state_guard
    //     .nodes
    //     .iter_mut()
    //     // Only direct games to online games
    //     .filter(|n| n.socket.online.load(Ordering::SeqCst))
    //     // Reserve some machines for the on-site cabinets
    //     .filter(|n| reserved == n.reserved)
    //     .sorted_by_key(|n| {
    //         let same_region = if region == Some(n.region.as_str()) {
    //             1
    //         } else {
    //             10
    //         };
    //         // give preference to the users preferred region
    //         (n.players.len() + 1) * same_region
    //     })
    //     .next() // Get the first with the fewest players
    //     .ok_or_else(|| {
    //         warn!("No nodes available");
    //         Status::ServiceUnavailable
    //     })?;

    // let addr = Address::from_bech32(address).map_err(|_| Status::BadRequest)?;

    // let player = Player::new(&addr).map_err(|_| Status::BadRequest)?;
    // let (player_utxo, player_utxo_datum_hex) =
    //     node.add_player(player, addr).await.map_err(|e| {
    //         warn!("failed to add player {:?}", e);
    //         Status::InternalServerError
    //     })?;

    // // TODO: move this to the frontend to lookup
    // // TODO: This is hard coded because our offline nodes have them in the initial-utxo
    // let script_ref =
    //     "0000000000000000000000000000000000000000000000000000000000000000#0".to_string();
    // Ok(Json(NewGameResponse {
    //     ip: node.remote_connection.to_authority(),
    //     script_ref,
    //     admin_pkh: node.tx_builder.admin_pkh.to_string(),
    //     player_utxo,
    //     player_utxo_datum_hex,
    // }))
}
