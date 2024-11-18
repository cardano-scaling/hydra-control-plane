use rocket::{get, http::Status, State};
use rocket_errors::anyhow::Result;
use tracing::{error, info};

use crate::model::cluster::{ClusterState, NodeClient};

#[get("/start_game?<id>")]
pub async fn start_game(id: &str, state: &State<ClusterState>) -> Result<(), Status> {
    let node = state
        .get_node_by_id(id)
        .ok_or(Status::BadRequest)
        .inspect_err(|_| error!("failed to fetch node with id: {}", id))?;

    let node_id = node.metadata.name.clone().expect("node without a name");
    info!(id = node_id, "start game for node");

    let client = NodeClient::new(node, state.admin_sk.clone(), state.remote)
        .inspect_err(|err| error!("failed to connect to node: {}", err))
        .map_err(|_| Status::ServiceUnavailable)?;

    info!(id = node_id, "connected to node");

    client
        .start_game()
        .await
        .inspect_err(|err| error!("failed to submit start game tx: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
