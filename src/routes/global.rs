use rocket::{http::Status, serde::json::Json, State};

use crate::{model::node::NodeStats, MyState};

#[get("/global")]
pub async fn global(state: &State<MyState>) -> Result<Json<NodeStats>, Status> {
    let state_guard = state.state.state.read().await;
    let stats = state_guard
        .nodes
        .iter()
        .fold(None, |acc: Option<NodeStats>, node| {
            if let Some(acc) = acc {
                Some(acc.join(node.clone().stats))
            } else {
                Some(node.clone().stats)
            }
        })
        .ok_or(Status::InternalServerError)?;

    Ok(Json(stats))
}
