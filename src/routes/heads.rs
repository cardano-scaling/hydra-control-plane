use kube::api::ObjectMeta;
use rocket::{get, serde::json::Json, State};

use crate::model::cluster::ClusterState;

#[get("/heads")]
pub async fn heads(state: &State<ClusterState>) -> Json<Vec<ObjectMeta>> {
    let nodes = state
        .get_all_nodes()
        .iter()
        .map(|x| x.metadata.clone())
        .collect();

    Json(nodes)
}
