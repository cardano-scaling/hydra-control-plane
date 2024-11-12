use anyhow::Context;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CommandFailed {
    client_input: Value,
    state: Value,
    seq: u64,
    timestamp: String,
}

impl TryFrom<Value> for CommandFailed {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let client_input = value["clientInput"].clone();
        let state = value["state"].clone();
        let seq = value["seq"].as_u64().context("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().context("Invalid timestamp")?;

        Ok(CommandFailed {
            client_input,
            state,
            seq,
            timestamp: timestamp.to_string(),
        })
    }
}
