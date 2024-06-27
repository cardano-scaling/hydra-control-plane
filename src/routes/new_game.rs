use crate::{
    model::{node::Node, player::Player},
    MyState,
};
use pallas::{codec::minicbor::encode, ledger::primitives::conway::PlutusData};
use rocket::{http::Status, State};

#[get("/new_game?<pkh>")]
pub async fn new_game(pkh: &str, state: &State<MyState>) -> Result<String, Status> {
    let max_players: usize = state.config.max_players.try_into().unwrap_or_else(|_| 1);
    let mut state_guard = state.state.state.write().await;
    let node: &mut Node = state_guard
        .nodes
        .iter_mut()
        .find(|n| n.players.len() < max_players)
        .ok_or(Status::ServiceUnavailable)?;

    let player = Player::new(pkh);
    let game_state = player.initialize_state();
    let plutus_data: PlutusData = game_state.into();
    let mut buffer: Vec<u8> = Vec::new();
    encode(plutus_data, &mut buffer).map_err(|_| Status::InternalServerError)?;
    let cbor = hex::encode(&buffer);
    println!(
        "Player: {:?} | GameState: {:?} | Datum: {:?}",
        pkh,
        player.initialize_state(),
        cbor,
    );
    node.players.push(player.clone());

    Ok(format!(
        "Welcome, Player! pkh: {:?} | Your Datum: {:?}",
        pkh, cbor
    ))
}
