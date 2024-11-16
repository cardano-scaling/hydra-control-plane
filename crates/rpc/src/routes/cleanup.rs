use rocket::{get, http::Status, State};
use tracing::error;

use crate::model::cluster::{ClusterState, NodeClient};

#[get("/cleanup?<id>")]
pub async fn cleanup(id: &str, state: &State<ClusterState>) -> Result<(), Status> {
    let node = state.get_node_by_id(id).ok_or(Status::NotFound)?;

    let client = NodeClient::new(node, state.admin_sk.clone(), state.remote)
        .inspect_err(|err| error!("error connecting to node: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    client
        .end_game()
        .await
        .inspect_err(|err| error!("failed to end game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    client
        .cleanup_game()
        .await
        .inspect_err(|err| error!("failed to cleanup game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
