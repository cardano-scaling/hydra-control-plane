use hydra_control_plane_rpc::model::cluster::{ConnectionInfo, NodeClient};
use rocket::{http::Status, post, State};
use rocket_errors::anyhow::Result;
use tracing::error;

use crate::{guards::api_key::ApiKey, LocalState};

#[post("/game/start_game")]
pub async fn start_game(_api_key: ApiKey, state: &State<LocalState>) -> Result<(), Status> {
    let client = NodeClient::new(ConnectionInfo::local(), state.admin_key.clone());

    client
        .start_game()
        .await
        .inspect_err(|err| error!("failed to submit start game tx: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
