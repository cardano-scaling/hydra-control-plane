use crate::{model::player::Player, MyState};
use rocket::{http::Status, State};

#[get("/get_node?<pkh>")]
pub async fn get_node(pkh: &str, state: &State<MyState>) -> Result<String, Status> {
    let max_players: usize = state.config.max_players.try_into().unwrap_or_else(|_| 1);
    let mut nodes = state.nodes.lock().unwrap();
    let index = nodes
        .iter()
        .position(|node| node.players.len() < max_players)
        .ok_or(Status::ServiceUnavailable)?;

    if let Some(node) = nodes.get_mut(index) {
        node.listen();
        node.players.push(Player::new(pkh));
        Ok(node.uri.clone())
    } else {
        Err(Status::ServiceUnavailable)
    }
}
