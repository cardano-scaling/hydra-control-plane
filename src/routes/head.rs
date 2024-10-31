use rocket::{get, http::Status, serde::json::Json, State};

use crate::model::cluster::{ClusterState, Node};

#[get("/heads/<head_id>")]
pub async fn head(state: &State<ClusterState>, head_id: &str) -> Result<Json<Vec<Node>>, Status> {
    let node = state
        .get_node_by_id(head_id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(vec![node]))
}
