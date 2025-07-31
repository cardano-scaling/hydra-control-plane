use anyhow::{Context, Result};
use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SnapshotConfirmed {
    pub head_id: String,
    pub seq: u64,
    pub signatures: Vec<String>,
    pub confirmed_transactions: Vec<Vec<u8>>,
    pub snapshot_number: u64,
    pub utxo: Vec<UTxO>,
    pub timestamp: String,
}

impl TryFrom<Value> for SnapshotConfirmed {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .context("Invalid head_id")?
            .to_owned();
        let seq = value["seq"].as_u64().context("Invalid seq")?;
        let signatures = value["signatures"]
            .as_object()
            .context("invalid signatures object")?["multiSignature"]
            .as_array()
            .context("Invalid multiSignatures")?
            .iter()
            .map(|s| s.as_str().context("invalid str").map(|s| s.to_string()))
            .collect::<Result<Vec<String>>>()?;
        let snapshot = value["snapshot"].as_object().context("Invalid snapshot")?;

        let confirmed_transactions = snapshot["confirmed"]
            .as_array()
            .context("Invalid confirmedTransactions")?
            .iter()
            .map(|s| {
                s.as_str()
                    .context("invalid confirmedTransaction")
                    .and_then(|s| hex::decode(s).context("failed to hex decode"))
            })
            .collect::<Result<Vec<Vec<u8>>>>()?;
        let snapshot_number = snapshot["number"]
            .as_u64()
            .context("Invalid snapshotNumber")?;
        let utxo = snapshot["utxo"]
            .as_object()
            .context("Invalid utxo")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>>>()?;
        let timestamp = value["timestamp"].as_str().context("Invalid timestamp")?;

        Ok(SnapshotConfirmed {
            head_id: head_id.to_string(),
            seq,
            signatures,
            confirmed_transactions,
            snapshot_number,
            utxo,
            timestamp: timestamp.to_string(),
        })
    }
}
