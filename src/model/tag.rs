use anyhow::{anyhow, Result};
use std::str::FromStr;
#[derive(Debug)]
pub enum Tag {
    Greetings,
    PeerConnected,
    PeerDisconnected,
    PeerHandshakeFailure,
    HeadIsInitializing,
    Committed,
    HeadIsOpen,
    HeadIsClosed,
    HeadIsContested,
    ReadyToFanout,
    HeadIsAborted,
    HeadIsFinalized,
    TxValid,
    TxInvalid,
    SnapshotConfirmed,
    GetUTxOResponse,
    InvalidInput,
    PostTxOnChainFailed,
    CommandFailed,
    IgnoredHeadInitializing,
}

impl FromStr for Tag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Greetings" => Ok(Tag::Greetings),
            "PeerConnected" => Ok(Tag::PeerConnected),
            "PeerDisconnected" => Ok(Tag::PeerDisconnected),
            "PeerHandshakeFailure" => Ok(Tag::PeerHandshakeFailure),
            "HeadIsInitializing" => Ok(Tag::HeadIsInitializing),
            "Committed" => Ok(Tag::Committed),
            "HeadIsOpen" => Ok(Tag::HeadIsOpen),
            "HeadIsClosed" => Ok(Tag::HeadIsClosed),
            "HeadIsContested" => Ok(Tag::HeadIsContested),
            "ReadyToFanout" => Ok(Tag::ReadyToFanout),
            "HeadIsAborted" => Ok(Tag::HeadIsAborted),
            "HeadIsFinalized" => Ok(Tag::HeadIsFinalized),
            "TxValid" => Ok(Tag::TxValid),
            "TxInvalid" => Ok(Tag::TxInvalid),
            "SnapshotConfirmed" => Ok(Tag::SnapshotConfirmed),
            "GetUTxOResponse" => Ok(Tag::GetUTxOResponse),
            "InvalidInput" => Ok(Tag::InvalidInput),
            "PostTxOnChainFailed" => Ok(Tag::PostTxOnChainFailed),
            "CommandFailed" => Ok(Tag::CommandFailed),
            "IgnoredHeadInitializing" => Ok(Tag::IgnoredHeadInitializing),
            _ => Err(anyhow!("Invalid tag: {s}").into()),
        }
    }
}
