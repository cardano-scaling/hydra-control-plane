use anyhow::{Context, Result};
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug)]
pub struct PeerConnected {
    peer: String,
    timestamp: String,
    seq: u64,
}

impl TryFrom<Value> for PeerConnected {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let peer = value["peer"].as_str().context("Invalid peer")?.to_string();
        let timestamp = value["timestamp"]
            .as_str()
            .context("Invalid timestamp")?
            .to_owned();
        let seq = value["seq"].as_u64().context("Invalid seq")?;

        Ok(PeerConnected {
            peer: peer.to_string(),
            timestamp: timestamp.to_string(),
            seq,
        })
    }
}
