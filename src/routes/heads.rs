use rocket::{get, serde::json::Json, State};

use crate::model::cluster::{ClusterState, NodeSummary};

#[get("/heads")]
pub async fn heads(state: &State<ClusterState>) -> Json<Vec<NodeSummary>> {
    let nodes = state
        .get_all_nodes()
        .await
        .iter()
        .map(|s| NodeSummary(s.clone()))
        .collect::<Vec<NodeSummary>>();

    Json(nodes)
}
