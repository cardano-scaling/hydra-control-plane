use std::{error::Error, fmt};

use async_tungstenite::tungstenite::Message;
use serde_json::{map::Values, Value};

use super::{super::tag::Tag, messages::snapshot_confirmed::SnapshotConfirmed};

pub enum HydraMessage {
    HydraEvent(HydraEventMessage),
    Ping(Vec<u8>),
}

#[derive(Debug)]
pub enum HydraEventMessage {
    SnapshotConfirmed(SnapshotConfirmed),
    Unimplemented(Value),
}

impl TryFrom<Value> for HydraEventMessage {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tag = value["tag"]
            .as_str()
            .ok_or("Invalid tag")?
            .parse::<Tag>()
            .map_err(|_| "Invalid tag")?;

        match tag {
            Tag::SnapshotConfirmed => {
                let snapshot_confirmed = SnapshotConfirmed::try_from(value)?;
                Ok(HydraEventMessage::SnapshotConfirmed(snapshot_confirmed))
            }
            _ => Ok(HydraEventMessage::Unimplemented(value)),
        }
    }
}

impl TryFrom<Message> for HydraMessage {
    type Error = HydraMessageError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => {
                let json: Value = serde_json::from_str(&text)
                    .map_err(|err| HydraMessageError::JsonParseError(err))?;
                let event = HydraEventMessage::try_from(json)
                    .map_err(|_| HydraMessageError::UnsupportedTag("".to_string()))?;
                Ok(HydraMessage::HydraEvent(event))
            }
            Message::Ping(payload) => Ok(HydraMessage::Ping(payload)),
            _ => Err(HydraMessageError::UnsupportedMessageFormat),
        }
    }
}

#[derive(Debug)]
pub enum HydraMessageError {
    UnsupportedMessageFormat,
    UnsupportedTag(String),
    JsonParseError(serde_json::Error),
    InvalidTag,
}

impl Error for HydraMessageError {}

impl fmt::Display for HydraMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HydraMessageError::UnsupportedMessageFormat => write!(f, "Invalid message format"),
            HydraMessageError::UnsupportedTag(tag) => write!(f, "unsupported tag: {tag}"),
            HydraMessageError::InvalidTag => write!(f, "invalid tag field"),
            HydraMessageError::JsonParseError(err) => write!(f, "json parse error: {err}"),
        }
    }
}
