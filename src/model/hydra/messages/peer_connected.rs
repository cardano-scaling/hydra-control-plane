use std::error::Error;

use serde_json::Value;

#[derive(Debug)]
pub struct PeerConnected {
    peer: String,
    timestamp: String,
    seq: u64,
}

impl TryFrom<Value> for PeerConnected {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let peer = value["peer"].as_str().ok_or("Invalid peer")?.to_string();
        let timestamp = value["timestamp"]
            .as_str()
            .ok_or("Invalid timestamp")?
            .to_owned();
        let seq = value["seq"].as_u64().ok_or("Invalid seq")?;

        Ok(PeerConnected {
            peer: peer.to_string(),
            timestamp: timestamp.to_string(),
            seq,
        })
    }
}
