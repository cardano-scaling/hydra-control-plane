use crate::model::node::Node;
use crate::MyState;
use rocket::serde::json::Json;
use rocket::State;

#[get("/heads/<head_id>")]
pub async fn head(state: &State<MyState>, head_id: String) -> Json<Vec<Node>> {
    let state_guard = state.state.state.read().await;
    let nodes = state_guard
        .nodes
        .clone()
        .iter()
        .filter_map(|n| {
            if n.head_id == Some(head_id.clone()) {
                Some(n.clone())
            } else {
                None
            }
        })
        .collect::<Vec<Node>>();

    Json(nodes)
}
