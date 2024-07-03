use crate::{
    model::{node::Node, player::Player},
    MyState,
};
use pallas::ledger::addresses::Address;
use rocket::{http::Status, State};

#[get("/new_game?<address>")]
pub async fn new_game(address: &str, state: &State<MyState>) -> Result<String, Status> {
    let max_players: usize = state.config.max_players.try_into().unwrap_or_else(|_| 1);
    let mut state_guard = state.state.state.write().await;
    let node: &mut Node = state_guard
        .nodes
        .iter_mut()
        .find(|n| n.players.len() < max_players)
        .ok_or(Status::ServiceUnavailable)?;

    let addr = Address::from_bech32(address).map_err(|_| Status::BadRequest)?;
    let pkh: String = match addr {
        Address::Shelley(shelley) => shelley.payment().to_hex(),
        _ => return Err(Status::BadRequest),
    };

    let player = Player::new(pkh.as_str());
    let _ = node.add_player(player).await.map_err(|e| {
        println!("failed to add player {:?}", e);
        Status::InternalServerError
    })?;

    Ok(format!("Welcome, Player! address: {:?}", address))
}
