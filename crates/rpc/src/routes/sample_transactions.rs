use crate::model::{
    cluster::{ClusterState, NodeClient},
    hydra::messages::tx_valid::TxValid,
};
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
    id: &str,
    state: &State<ClusterState>,
) -> Result<Json<Vec<SampleTransaction>>, Status> {
    let node = state.get_node_by_id(id).ok_or(Status::NotFound)?;
    let client = NodeClient::new(node, state.admin_sk.clone(), true)
        .inspect_err(|err| error!("error connecting to node: {}", err))
        .map_err(|_| Status::InternalServerError)?;

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

impl From<TxValid> for SampleTransaction {
    fn from(value: TxValid) -> Self {
        Self {
            cbor: hex::encode(value.cbor),
            tx_id: value.tx_id,
        }
    }
}
