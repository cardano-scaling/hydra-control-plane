use anyhow::{Context, Result};
use derivative::Derivative;
use serde_json::Value;

use crate::model::hydra::utxo::UTxO;

#[allow(dead_code)]
#[allow(clippy::needless_lifetimes)]
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
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .context("Invalid head_id")?
            .to_owned();
        let party_obj = value["party"].as_object().context("Invalid party object")?;

        let party = hex::decode(party_obj["vkey"].as_str().context("Invalid vkey")?)?;
        let seq = value["seq"].as_u64().context("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().context("Invalid timestamp")?;
        let utxos = value["utxo"]
            .as_object()
            .context("Invalid UTxOs object")?
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>>>()?;

        Ok(Committed {
            head_id: head_id.to_string(),
            party,
            seq,
            timestamp: timestamp.to_string(),
            utxos,
        })
    }
}
