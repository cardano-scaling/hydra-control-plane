use rand::seq::SliceRandom;

use crate::model::{
    cluster::{ClusterState, NodeClient},
    hydra::messages::tx_valid::TxValid,
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
            .get_all_nodes()
            .choose(&mut thread_rng())
            .ok_or(Status::NotFound)?
            .to_owned(),
    };
    let client = NodeClient::new(node, state.admin_sk.clone(), state.remote)
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
