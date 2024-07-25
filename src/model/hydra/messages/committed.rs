use std::error::Error;

use derivative::Derivative;
use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

#[allow(dead_code)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Committed {
    head_id: String,
    #[derivative(Debug(format_with = "crate::model::format_hex"))]
    party: Vec<u8>,
    seq: u64,
    timestamp: String,
    utxos: Vec<UTxO>,
}

impl TryFrom<Value> for Committed {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .ok_or("Invalid head_id")?
            .to_owned();
        let party_obj = value["party"].as_object().ok_or("Invalid party object")?;

        let party = hex::decode(party_obj["vkey"].as_str().ok_or("Invalid vkey")?)?;
        let seq = value["seq"].as_u64().ok_or("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().ok_or("Invalid timestamp")?;
        let utxos = value["utxo"]
            .as_object()
            .ok_or("Invalid UTxOs object")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>, Box<dyn std::error::Error>>>()?;

        Ok(Committed {
            head_id: head_id.to_string(),
            party,
            seq,
            timestamp: timestamp.to_string(),
            utxos,
        })
    }
}
