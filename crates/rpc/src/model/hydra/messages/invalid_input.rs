use anyhow::{bail, Context, Result};
use derivative::Derivative;
use serde_json::Value;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct InvalidInput {
    pub reason: String,
    pub input: String,
    pub seq: u64,
    pub timestamp: String,
}

impl TryFrom<Value> for InvalidInput {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tag = value
            .get("tag")
            .context("Missing tag")?
            .as_str()
            .context("Invalid tag")?;

        if tag != "InvalidInput" {
            bail!("Incorrect tag for InvalidInput");
        }

        let reason = value
            .get("reason")
            .context("Missing reason")?
            .as_str()
            .context("Invalid reason")?
            .to_string();

        let input = value
            .get("input")
            .context("Missing input")?
            .as_str()
            .context("Invalid input")?
            .to_string();

        let seq = value
            .get("seq")
            .context("Missings seq")?
            .as_u64()
            .context("Invalid seq")?;

        let timestamp = value
            .get("timestamp")
            .context("Missing timestamp")?
            .as_str()
            .context("Invalid timestamp")?
            .to_string();

        Ok(InvalidInput {
            reason,
            input,
            seq,
            timestamp,
        })
    }
}
