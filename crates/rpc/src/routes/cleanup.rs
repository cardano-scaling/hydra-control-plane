use rocket::{http::Status, post, State};
use tracing::error;

use crate::{
    guards::api_key::ApiKey,
    model::cluster::{ClusterState, NodeClient},
};

#[post("/cleanup?<id>")]
pub async fn cleanup(
    id: &str,
    _api_key: ApiKey,
    state: &State<ClusterState>,
) -> Result<(), Status> {
    let node = state.get_node_by_id(id).ok_or(Status::NotFound)?;

    let client = NodeClient::new(node, state.admin_sk.clone(), state.remote, state.network)
        .inspect_err(|err| error!("error connecting to node: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    client
        .cleanup_game()
        .await
        .inspect_err(|err| error!("failed to cleanup game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
