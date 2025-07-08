use anyhow::{Context, Result};
use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Greetings {
    head_status: String,
    hydra_node_version: String,
    me: Vec<u8>,
    snapshot_utxos: Vec<UTxO>,
}

impl TryFrom<Value> for Greetings {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_status = value["headStatus"]
            .as_str()
            .context("Invalid head_status")?
            .to_owned();
        let hydra_node_version = value["hydraNodeVersion"]
            .as_str()
            .context("Invalid hydra_node_version")?
            .to_owned();
        let me_obj = value["me"].as_object().context("Invalid me object")?;
        let me = hex::decode(me_obj["vkey"].as_str().context("Invalid me vkey")?)?;
        let snapshot_utxos = value["snapshotUtxo"]
            .as_object()
            .context("Invalid snapshotUtxo object")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>>>()?;
        Ok(Greetings {
            head_status: head_status.to_string(),
            hydra_node_version: hydra_node_version.to_string(),
            me,
            snapshot_utxos,
        })
    }
}
