use anyhow::Context;
use serde_json::Value;

pub mod command_failed;
pub mod committed;
pub mod greetings;
pub mod head_is_aborted;
pub mod head_is_initializing;
pub mod head_is_open;
pub mod init;
pub mod invalid_input;
pub mod new_tx;
pub mod peer_connected;
pub mod peer_disconnected;
pub mod snapshot_confirmed;
pub mod tx_valid;

#[derive(Debug, Eq, PartialEq)]
pub struct Transaction {
    pub cbor: Vec<u8>,
    pub description: String,
    pub tx_id: String,
    pub tx_type: String,
}

impl TryFrom<&Value> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let cbor = hex::decode(
            value
                .get("cborHex")
                .context("missing cborHex field")?
                .as_str()
                .context("invalid cborHex field")?,
        )?;
        let description = value
            .get("description")
            .context("missing description field")?
            .as_str()
            .context("invalid description value")?
            .to_owned();
        let tx_id = value
            .get("txId")
            .context("missing txId field")?
            .as_str()
            .context("invalid txId field")?
            .to_owned();
        let tx_type = value
            .get("type")
            .context("missing type field")?
            .as_str()
            .context("invalid type field")?
            .to_owned();

        Ok(Transaction {
            cbor,
            description,
            tx_id,
            tx_type,
        })
    }
}
