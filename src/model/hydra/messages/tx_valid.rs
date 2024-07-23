use std::error::Error;

use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TxValid {
    pub head_id: String,
    pub seq: u64,
    pub timestamp: String,
    pub cbor: Vec<u8>,
    pub descrption: String,
    pub tx_id: Vec<u8>,
    pub tx_type: String,
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

        let cbor = match transaction["cborHex"].as_str() {
            Some(cbor) => hex::decode(cbor)?,
            None => return Err("Invalid cbor".into()),
        };

        let descrption = transaction["description"]
            .as_str()
            .ok_or("Invalid descrption")?
            .to_owned();

        let tx_id = match transaction["txId"].as_str() {
            Some(tx_id) => hex::decode(tx_id)?,
            None => return Err("Invalid txId".into()),
        };

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
