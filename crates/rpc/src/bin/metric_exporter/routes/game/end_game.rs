use hydra_control_plane_rpc::model::cluster::{ConnectionInfo, NodeClient};
use rocket::{http::Status, post, State};
use tracing::error;

use crate::LocalState;

#[post("/game/end_game")]
pub async fn end_game(state: &State<LocalState>) -> Result<(), Status> {
    let client = NodeClient::new(
        ConnectionInfo::local(),
        state.admin_key.clone(),
        state.network,
    );

    // TODO: we need to take in the "end state" of the game. Currently, we are always aborting
    client
        .end_game()
        .await
        .inspect_err(|err| error!("failed to end game: {}", err))
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
