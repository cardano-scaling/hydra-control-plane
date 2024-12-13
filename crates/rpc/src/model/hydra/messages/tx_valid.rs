use anyhow::Context;
use serde_json::Value;

use super::Transaction;

#[derive(Debug, Eq, PartialEq)]
pub struct TxValid {
    pub head_id: String,
    pub seq: u64,
    pub transaction: Transaction,
    pub timestamp: String,
}

impl TryFrom<Value> for TxValid {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value
            .get("headId")
            .context("missing head_id")?
            .as_str()
            .context("Invalid head_id")?
            .to_owned();
        let seq = value
            .get("seq")
            .context("missing seq")?
            .as_u64()
            .context("Invalid seq")?;
        let timestamp = value
            .get("timestamp")
            .context("missing timestamp")?
            .as_str()
            .context("Invalid timestamp")?;
        let transaction: Transaction = value
            .get("transaction")
            .context("missing transaction")?
            .try_into()
            .context("Invalid transaction")?;

        Ok(TxValid {
            head_id: head_id.to_string(),
            seq,
            transaction,
            timestamp: timestamp.to_string(),
        })
    }
}
