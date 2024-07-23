use std::error::Error;

use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Greetings {
    head_status: String,
    hydra_node_version: String,
    me: Vec<u8>,
    seq: u64,
    snapshot_utxos: Vec<UTxO>,
    timestamp: String,
}

impl TryFrom<Value> for Greetings {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_status = value["headStatus"]
            .as_str()
            .ok_or("Invalid head_status")?
            .to_owned();
        let hydra_node_version = value["hydraNodeVersion"]
            .as_str()
            .ok_or("Invalid hydra_node_version")?
            .to_owned();
        let me_obj = value["me"].as_object().ok_or("Invalid me object")?;
        let me = hex::decode(me_obj["vkey"].as_str().ok_or("Invalid me vkey")?)?;
        let seq = value["seq"].as_u64().ok_or("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().ok_or("Invalid timestamp")?;
        let snapshot_utxos = value["snapshotUtxo"]
            .as_object()
            .ok_or("Invalid snapshotUtxo object")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>, Box<dyn std::error::Error>>>()?;

        Ok(Greetings {
            head_status: head_status.to_string(),
            hydra_node_version: hydra_node_version.to_string(),
            me,
            seq,
            snapshot_utxos,
            timestamp: timestamp.to_string(),
        })
    }
}
