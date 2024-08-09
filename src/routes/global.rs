use std::time::Duration;

use rocket::{http::Status, serde::json::Json, State};

use crate::{model::node::NodeStats, MyState};

#[get("/global")]
pub async fn global(state: &State<MyState>) -> Result<Json<NodeStats>, Status> {
    let state_guard = state.state.state.read().await;
    let stats = state_guard
        .nodes
        .iter()
        .fold(NodeStats::new(), |acc: NodeStats, node| {
            acc.join(
                node.clone().stats,
                node.players
                    .iter()
                    .filter(|p| p.is_expired(Duration::from_secs(5)))
                    .count(),
            )
        });

    Ok(Json(stats))
}
