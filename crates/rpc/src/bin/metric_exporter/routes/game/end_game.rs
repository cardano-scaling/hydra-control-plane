use hydra_control_plane_rpc::model::cluster::{ConnectionInfo, NodeClient};
use rocket::{http::Status, post, State};
use tracing::error;

use crate::{guards::api_key::ApiKey, LocalState};

#[post("/game/end_game")]
pub async fn end_game(_api_key: ApiKey, state: &State<LocalState>) -> Result<(), Status> {
    let client = NodeClient::new(ConnectionInfo::local(), state.admin_key.clone());

    // TODO: we need to take in the "end state" of the game. Currently, we are always aborting
    client
        .end_game()
        .await
        .inspect_err(|err| error!("failed to end game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
