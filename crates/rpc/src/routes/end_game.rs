use rocket::{http::Status, post, State};
use tracing::error;

use crate::{
    guards::api_key::ApiKey,
    model::cluster::{ClusterState, NodeClient},
};

#[post("/end_game?<id>")]
pub async fn end_game(
    id: &str,
    _api_key: ApiKey,
    state: &State<ClusterState>,
) -> Result<(), Status> {
    let node = state.get_node_by_id(id).ok_or(Status::NotFound)?;

    let client = NodeClient::new(node, state.admin_sk.clone(), state.remote)
        .inspect_err(|err| error!("error connecting to node: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    // TODO: we need to take in the "end state" of the game. Currently, we are always aborting
    client
        .end_game()
        .await
        .inspect_err(|err| error!("failed to end game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
