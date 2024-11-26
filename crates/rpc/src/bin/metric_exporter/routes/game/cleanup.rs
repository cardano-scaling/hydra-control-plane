use hydra_control_plane_rpc::model::cluster::{ConnectionInfo, NodeClient};
use rocket::{http::Status, post, State};
use tracing::error;

use crate::LocalState;

#[post("/game/cleanup")]
pub async fn cleanup(state: &State<LocalState>) -> Result<(), Status> {
    let client = NodeClient::new(ConnectionInfo::local(), state.admin_key.clone());

    client
        .cleanup_game()
        .await
        .inspect_err(|err| error!("failed to cleanup game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
