use rocket::{get, serde::json::Json, State};

use crate::{model::node::NodeSummary, MyState};

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
