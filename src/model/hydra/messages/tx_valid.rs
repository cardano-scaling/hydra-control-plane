use anyhow::Context;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TxValid {
    pub head_id: String,
    pub seq: u64,
    pub timestamp: String,
    pub tx_id: String,
}

impl TryFrom<Value> for TxValid {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .context("Invalid head_id")?
            .to_owned();
        let seq = value["seq"].as_u64().context("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().context("Invalid timestamp")?;

        let tx_id = value["transactionId"]
            .as_str()
            .context("Invalid txId")?
            .to_owned();

        Ok(TxValid {
            head_id: head_id.to_string(),
            seq,
            timestamp: timestamp.to_string(),
            tx_id,
        })
    }
}
