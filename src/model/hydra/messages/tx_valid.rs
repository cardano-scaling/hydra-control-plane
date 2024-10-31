use anyhow::Context;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TxValid {
    pub head_id: String,
    pub seq: u64,
    pub timestamp: String,
    pub cbor: Vec<u8>,
    pub descrption: String,
    pub tx_id: String,
    pub tx_type: String,
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
        let transaction = value["transaction"]
            .as_object()
            .context("Invalid transaction")?;

        let cbor = hex::decode(transaction["cborHex"].as_str().context("invalid cbor")?)?;

        let descrption = transaction["description"]
            .as_str()
            .context("Invalid descrption")?
            .to_owned();

        let tx_id = transaction["txId"]
            .as_str()
            .context("Invalid txId")?
            .to_owned();

        let tx_type = transaction["type"]
            .as_str()
            .context("Invalid txType")?
            .to_owned();

        Ok(TxValid {
            head_id: head_id.to_string(),
            seq,
            timestamp: timestamp.to_string(),
            cbor,
            descrption: descrption.to_string(),
            tx_id,
            tx_type: tx_type.to_string(),
        })
    }
}
