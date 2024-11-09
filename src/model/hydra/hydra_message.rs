use std::{error::Error, fmt};

use anyhow::{Context, Result};
use async_tungstenite::tungstenite::Message;
use serde_json::Value;

use super::messages::{
    command_failed::CommandFailed, committed::Committed, greetings::Greetings,
    head_is_initializing::HeadIsInitializing, head_is_open::HeadIsOpen,
    invalid_input::InvalidInput, peer_connected::PeerConnected,
    peer_disconnected::PeerDisconnected, snapshot_confirmed::SnapshotConfirmed, tx_valid::TxValid,
};

#[derive(Debug)]
pub enum HydraMessage {
    HydraEvent(HydraEventMessage),
    Ping(Vec<u8>),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum HydraData {
    Received {
        message: HydraEventMessage,
        authority: String,
    },
    Send(String),
}

#[allow(dead_code)]
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
    CommandFailed(CommandFailed),
    InvalidInput(InvalidInput),
    Unimplemented(Value),
}

impl TryFrom<Value> for HydraEventMessage {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tag = value["tag"].as_str().context("Invalid tag")?;
        match tag {
            "SnapshotConfirmed" => {
                SnapshotConfirmed::try_from(value).map(HydraEventMessage::SnapshotConfirmed)
            }
            "TxValid" => TxValid::try_from(value).map(HydraEventMessage::TxValid),
            "PeerConnected" => PeerConnected::try_from(value).map(HydraEventMessage::PeerConnected),
            "PeerDisconnected" => {
                PeerDisconnected::try_from(value).map(HydraEventMessage::PeerDisconnected)
            }
            "HeadIsInitializing" => {
                HeadIsInitializing::try_from(value).map(HydraEventMessage::HeadIsInitializing)
            }
            "HeadIsOpen" => HeadIsOpen::try_from(value).map(HydraEventMessage::HeadIsOpen),
            "Committed" => Committed::try_from(value).map(HydraEventMessage::Committed),
            "Greetings" => Greetings::try_from(value).map(HydraEventMessage::Greetings),
            "CommandFailed" => CommandFailed::try_from(value).map(HydraEventMessage::CommandFailed),
            "InvalidInput" => InvalidInput::try_from(value).map(HydraEventMessage::InvalidInput),
            _ => Ok(HydraEventMessage::Unimplemented(value)),
        }
    }
}

impl TryFrom<Message> for HydraMessage {
    type Error = HydraMessageError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => {
                let json: Value =
                    serde_json::from_str(&text).map_err(HydraMessageError::JsonParseError)?;
                let event = HydraEventMessage::try_from(json)
                    .map_err(|e| HydraMessageError::UnknownError(e.to_string()))?;
                Ok(HydraMessage::HydraEvent(event))
            }
            Message::Ping(payload) => Ok(HydraMessage::Ping(payload)),
            _ => Err(HydraMessageError::UnsupportedMessageFormat),
        }
    }
}

#[allow(dead_code)]
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
