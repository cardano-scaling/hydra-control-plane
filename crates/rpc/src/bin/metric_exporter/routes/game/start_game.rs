use hydra_control_plane_rpc::model::cluster::NodeClient;
use rocket::{http::Status, post, State};
use rocket_errors::anyhow::Result;
use tracing::error;

use crate::LocalState;

#[post("/game/start_game")]
pub async fn start_game(state: &State<LocalState>) -> Result<(), Status> {
    let client = NodeClient::new(state.hydra.clone(), state.admin_key.clone(), state.network);

    client
        .start_game()
        .await
        .inspect_err(|err| error!("failed to submit start game tx: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
