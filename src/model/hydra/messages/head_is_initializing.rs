use anyhow::{Context, Result};
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug)]
pub struct HeadIsInitializing {
    head_id: String,
    parties: Vec<Vec<u8>>,
    seq: u64,
    timestamp: String,
}

impl TryFrom<Value> for HeadIsInitializing {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .context("Invalid head_id")?
            .to_owned();
        let parties_arr = value["parties"].as_array().context("Invalid parties")?;
        let parties = parties_arr
            .into_iter()
            .map(|party| {
                party
                    .as_object()
                    .and_then(|m| m["vkey"].as_str())
                    .context("missing vkey")
                    .and_then(|vkey| hex::decode(vkey).context("invalid hex"))
                    .context("invalid vkey")
            })
            .collect::<Result<Vec<Vec<u8>>>>()?;
        let seq = value["seq"].as_u64().context("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().context("Invalid timestamp")?;

        Ok(HeadIsInitializing {
            head_id: head_id.to_string(),
            parties,
            seq,
            timestamp: timestamp.to_string(),
        })
    }
}
