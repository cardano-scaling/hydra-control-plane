use std::error::Error;

use serde_json::Value;

#[derive(Debug)]
pub struct TxValid {
    head_id: String,
    seq: u64,
    timestamp: String,
    cbor: Vec<u8>,
    descrption: String,
    tx_id: Vec<u8>,
    tx_type: String,
}

impl TryFrom<Value> for TxValid {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .ok_or("Invalid head_id")?
            .to_owned();
        let seq = value["seq"].as_u64().ok_or("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().ok_or("Invalid timestamp")?;
        let transaction = value["transaction"]
            .as_object()
            .ok_or("Invalid transaction")?;

        let cbor = hex::decode(transaction["cborHex"].as_str().ok_or("Invalid cbor")?)?;
        let descrption = transaction["description"]
            .as_str()
            .ok_or("Invalid descrption")?
            .to_owned();
        let tx_id = hex::decode(transaction["txId"].as_str().ok_or("Invalid txId")?)?;
        let tx_type = transaction["type"]
            .as_str()
            .ok_or("Invalid txType")?
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
