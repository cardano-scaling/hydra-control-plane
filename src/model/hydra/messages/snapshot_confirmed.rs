use std::error::Error;

use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

pub struct SnapshotConfirmed {
    head_id: String,
    seq: u64,
    signatures: Vec<String>,
    confirmed_transactions: Vec<String>,
    snapshot_number: u64,
    utxo: Vec<UTxO>,
    timestamp: String,
}

impl TryFrom<Value> for SnapshotConfirmed {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .ok_or("Invalid head_id")?
            .to_owned();
        let seq = value["seq"].as_u64().ok_or("Invalid seq")?;
        let signatures = value["signatures"]
            .as_array()
            .ok_or("Invalid signatures")?
            .iter()
            .map(|s| s.as_str().ok_or("Invalid signature").map(|s| s.to_string()))
            .collect::<Result<Vec<String>, &str>>()?;
        let confirmed_transactions = value["confirmed_transactions"]
            .as_array()
            .ok_or("Invalid confirmed_transactions")?
            .iter()
            .map(|s| {
                s.as_str()
                    .ok_or("Invalid confirmed_transaction")
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>, &str>>()?;
        let snapshot_number = value["snapshot_number"]
            .as_u64()
            .ok_or("Invalid snapshot_number")?;
        let utxo = value["utxo"]
            .as_array()
            .ok_or("Invalid utxo")?
            .iter()
            .map(|s| UTxO::try_from(s.clone()))
            .collect()?;
        let timestamp = value["timestamp"].as_str().ok_or("Invalid timestamp")?;

        Ok(SnapshotConfirmed {
            head_id: head_id.to_string(),
            seq,
            signatures,
            confirmed_transactions,
            snapshot_number,
            utxo: vec![],
            timestamp: timestamp.to_string(),
        })
    }
}
