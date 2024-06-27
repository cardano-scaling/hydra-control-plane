use std::{error::Error, fmt};

use async_tungstenite::tungstenite::Message;
use serde_json::Value;

use super::{
    super::tag::Tag,
    messages::{
        committed::Committed, greetings::Greetings, head_is_initializing::HeadIsInitializing,
        head_is_open::HeadIsOpen, peer_connected::PeerConnected,
        peer_disconnected::PeerDisconnected, snapshot_confirmed::SnapshotConfirmed,
        tx_valid::TxValid,
    },
};

pub enum HydraMessage {
    HydraEvent(HydraEventMessage),
    Ping(Vec<u8>),
}

#[derive(Debug)]
pub enum HydraData {
    Received {
        message: HydraEventMessage,
        authority: String,
    },
    Sent(String),
}

#[derive(Debug)]
pub enum HydraEventMessage {
    SnapshotConfirmed(SnapshotConfirmed),
    TxValid(TxValid),
    PeerConnected(PeerConnected),
    PeerDisconnected(PeerDisconnected),
    HeadIsInitializing(HeadIsInitializing),
    HeadIsOpen(HeadIsOpen),
    Committed(Committed),
    Greetings(Greetings),
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
                SnapshotConfirmed::try_from(value).map(HydraEventMessage::SnapshotConfirmed)
            }
            Tag::TxValid => TxValid::try_from(value).map(HydraEventMessage::TxValid),
            Tag::PeerConnected => {
                PeerConnected::try_from(value).map(HydraEventMessage::PeerConnected)
            }
            Tag::PeerDisconnected => {
                PeerDisconnected::try_from(value).map(HydraEventMessage::PeerDisconnected)
            }
            Tag::HeadIsInitializing => {
                HeadIsInitializing::try_from(value).map(HydraEventMessage::HeadIsInitializing)
            }
            Tag::HeadIsOpen => HeadIsOpen::try_from(value).map(HydraEventMessage::HeadIsOpen),
            Tag::Committed => Committed::try_from(value).map(HydraEventMessage::Committed),
            Tag::Greetings => Greetings::try_from(value).map(HydraEventMessage::Greetings),
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
                    .map_err(|e| HydraMessageError::UnknownError(e.to_string()))?;
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
    UnknownError(String),
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
            HydraMessageError::UnknownError(err) => write!(f, "unknown error: {err}"),
        }
    }
}
