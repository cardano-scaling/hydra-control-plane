use std::error::Error;

use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

#[derive(Debug)]
pub struct SnapshotConfirmed {
    pub head_id: String,
    pub seq: u64,
    pub signatures: Vec<String>,
    pub confirmed_transactions: Vec<String>,
    pub snapshot_number: u64,
    pub utxo: Vec<UTxO>,
    pub timestamp: String,
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
            .as_object()
            .ok_or("invalid signatures object")?["multiSignature"]
            .as_array()
            .ok_or("Invalid multiSignatures")?
            .iter()
            .map(|s| s.as_str().ok_or("Invalid signature").map(|s| s.to_string()))
            .collect::<Result<Vec<String>, &str>>()?;
        let snapshot = value["snapshot"].as_object().ok_or("Invalid snapshot")?;

        let confirmed_transactions = snapshot["confirmedTransactions"]
            .as_array()
            .ok_or("Invalid confirmedTransactions")?
            .iter()
            .map(|s| {
                s.as_str()
                    .ok_or("Invalid transaction")
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>, &str>>()?;
        let snapshot_number = snapshot["snapshotNumber"]
            .as_u64()
            .ok_or("Invalid snapshotNumber")?;
        let utxo = snapshot["utxo"]
            .as_object()
            .ok_or("Invalid utxo")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>, Box<dyn std::error::Error>>>()?;
        let timestamp = value["timestamp"].as_str().ok_or("Invalid timestamp")?;

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
