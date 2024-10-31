use rocket::{get, serde::json::Json, State};

use crate::model::cluster::{ClusterState, HydraDoomNodeSpec};

#[get("/heads")]
pub async fn heads(state: &State<ClusterState>) -> Json<Vec<HydraDoomNodeSpec>> {
    let nodes = state
        .get_all_nodes()
        .iter()
        .map(|x| x.spec.clone())
        .collect();

    Json(nodes)
}
