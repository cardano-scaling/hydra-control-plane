use anyhow::{Context, Result};
use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

use super::Transaction;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SnapshotConfirmed {
    pub head_id: String,
    pub seq: u64,
    pub signatures: Vec<String>,
    pub confirmed_transactions: Vec<Transaction>,
    pub snapshot_number: u64,
    pub utxo: Vec<UTxO>,
    pub timestamp: String,
}

impl TryFrom<Value> for SnapshotConfirmed {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value
            .get("headId")
            .context("missing headId")?
            .as_str()
            .context("invalid head_id")?
            .to_owned();
        let seq = value
            .get("seq")
            .context("missing seq")?
            .as_u64()
            .context("invalid seq")?;
        let signatures = value
            .get("signatures")
            .context("missing signatures")?
            .as_object()
            .context("invalid signatures object")?
            .get("multiSignature")
            .context("missing multiSignature")?
            .as_array()
            .context("invalid multiSignature")?
            .iter()
            .map(|s| s.as_str().context("invalid str").map(|s| s.to_string()))
            .collect::<Result<Vec<String>>>()?;
        let snapshot = value
            .get("snapshot")
            .context("missing snapshot")?
            .as_object()
            .context("invalid snapshot")?;

        let confirmed_transactions = snapshot
            .get("confirmed")
            .context("missing confirmed")?
            .as_array()
            .context("invalid confirmed")?
            .iter()
            .map(|tx| tx.try_into().context("failed to decode transaction"))
            .collect::<Result<Vec<Transaction>>>()?;

        let snapshot_number = snapshot
            .get("number")
            .context("missing number")?
            .as_u64()
            .context("invalid snapshotNumber")?;

        let utxo = snapshot
            .get("utxo")
            .context("missing utxo")?
            .as_object()
            .context("invalid utxo")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>>>()?;
        let timestamp = value
            .get("timestamp")
            .context("missing timestamp")?
            .as_str()
            .context("invalid timestamp")?;

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
