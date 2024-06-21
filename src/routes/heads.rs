use crate::{model::node::Node, MyState};
use rocket::serde::json::Json;
use rocket::State;

#[get("/heads")]
pub async fn heads(state: &State<MyState>) -> Json<Vec<Node>> {
    let state_guard = state.state.state.read().await;
    let nodes = state_guard.nodes.clone();

    Json(nodes)
}
