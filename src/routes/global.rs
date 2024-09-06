use std::time::Duration;

use rocket::{get, http::Status, serde::json::Json, State};

use crate::{model::node::NodeStats, MyState};

#[get("/global")]
pub async fn global(state: &State<MyState>) -> Result<Json<NodeStats>, Status> {
    let mut state_guard = state.state.state.write().await;
    let stats = state_guard
        .nodes
        .iter_mut()
        .fold(NodeStats::new(), |acc: NodeStats, node| {
            if node.socket.online.load(std::sync::atomic::Ordering::SeqCst) {
                node.stats.online_nodes = 1;
                node.stats.offline_nodes = 0;
            } else {
                node.stats.offline_nodes = 1;
                node.stats.online_nodes = 0;
            }
            acc.join(
                node.clone().stats,
                node.players
                    .iter()
                    .filter(|p| !p.is_expired(Duration::from_secs(5)) && p.utxo.is_some())
                    .count(),
            )
        });

    Ok(Json(stats))
}
