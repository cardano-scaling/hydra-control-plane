use rocket::{get, http::Status, serde::json::Json, State};

use crate::model::cluster::{ClusterState, HydraDoomNodeSpec};

#[get("/heads/<head_id>")]
pub async fn head(
    state: &State<ClusterState>,
    head_id: &str,
) -> Result<Json<Vec<HydraDoomNodeSpec>>, Status> {
    let node = state
        .get_node_by_id(head_id)
        .map(|x| x.spec.clone())
        .ok_or(Status::NotFound)?;

    Ok(Json(vec![node]))
}
