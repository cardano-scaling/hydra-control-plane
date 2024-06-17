use std::{error::Error, fmt};

use async_tungstenite::tungstenite::Message;
use serde_json::Value;

use super::super::tag::Tag;

pub enum HydraMessage {
    HydraEvent(HydraEventMessage),
    Ping(Vec<u8>),
}

// TODO: this should be an enum, which each variant being a struct that represents a different message schema
pub struct HydraEventMessage {
    pub tag: Tag,
    pub data: Value,
}

impl TryFrom<Message> for HydraMessage {
    type Error = HydraMessageError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => {
                let json: Value = serde_json::from_str(&text)
                    .map_err(|err| HydraMessageError::JsonParseError(err))?;
                let tag_str = json["tag"].as_str().ok_or(HydraMessageError::InvalidTag)?;
                let tag = tag_str
                    .parse::<Tag>()
                    .map_err(|_| HydraMessageError::UnsupportedTag(tag_str.to_string()))?;

                Ok(HydraMessage::HydraEvent(HydraEventMessage {
                    tag,
                    data: json.clone(),
                }))
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
