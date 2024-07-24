use std::error::Error;

use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

#[allow(dead_code)]
#[derive(Debug)]
pub struct HeadIsOpen {
    pub head_id: String,
    pub seq: u64,
    pub utxos: Vec<UTxO>,
    pub timestamp: String,
}

impl TryFrom<Value> for HeadIsOpen {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .ok_or("Invalid head_id")?
            .to_owned();
        let seq = value["seq"].as_u64().ok_or("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().ok_or("Invalid timestamp")?;
        let utxos = value["utxo"]
            .as_object()
            .ok_or("Invalid UTxOs object")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>, Box<dyn std::error::Error>>>()?;

        Ok(HeadIsOpen {
            head_id: head_id.to_string(),
            seq,
            utxos,
            timestamp: timestamp.to_string(),
        })
    }
}
