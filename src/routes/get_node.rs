use crate::{model::player::Player, MyState};
use rocket::{http::Status, State};

#[get("/get_node?<pkh>")]
pub async fn get_node(pkh: &str, state: &State<MyState>) -> Result<String, Status> {
    let max_players: usize = state.config.max_players.try_into().unwrap_or_else(|_| 1);

    Ok("Hello, World!".to_string())
}
