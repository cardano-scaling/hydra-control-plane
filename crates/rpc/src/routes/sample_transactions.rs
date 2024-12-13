use crate::model::{
    cluster::{ClusterState, ConnectionInfo, NodeClient},
    hydra::messages::Transaction,
};
use rand::thread_rng;
use rocket::{get, http::Status, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
pub struct SampleTransaction {
    cbor: String,
    tx_id: String,
}

#[get("/sample_transactions?<count>&<id>")]
pub async fn sample_transactions(
    count: usize,
    id: Option<&str>,
    state: &State<ClusterState>,
) -> Result<Json<Vec<SampleTransaction>>, Status> {
    let node = match id {
        Some(id) => state.get_node_by_id(id).ok_or(Status::NotFound)?,
        None => state
            .select_random_node_with_active_game()
            .map_err(|_| Status::NotFound)?,
    };

    let (local, remote) =
        ConnectionInfo::from_resource(node.status.as_ref().ok_or(Status::InternalServerError)?)
            .map_err(|_| Status::InternalServerError)?;

    let client = NodeClient::new(
        if state.remote { remote } else { local },
        state.admin_sk.clone(),
        state.network,
    );

    let transactions = client
        .sample_txs(count)
        .await
        .inspect_err(|err| error!("error sampling transactions: {}", err))
        .map_err(|_| Status::InternalServerError)?
        .into_iter()
        .map(|x| x.into())
        .collect::<Vec<SampleTransaction>>();

    Ok(Json(transactions))
}

impl From<Transaction> for SampleTransaction {
    fn from(value: Transaction) -> Self {
        Self {
            cbor: hex::encode(value.cbor),
            tx_id: value.tx_id,
        }
    }
}
