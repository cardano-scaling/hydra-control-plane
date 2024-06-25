use crate::model::node::NodeSummary;
use crate::MyState;
use rocket::serde::json::Json;
use rocket::State;

#[get("/heads")]
pub async fn heads(state: &State<MyState>) -> Json<Vec<NodeSummary>> {
    let state_guard = state.state.state.read().await;
    let nodes = state_guard
        .nodes
        .clone()
        .iter()
        .map(|s| NodeSummary(s.clone()))
        .collect::<Vec<NodeSummary>>();

    Json(nodes)
}
