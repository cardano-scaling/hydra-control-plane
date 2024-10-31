use std::{
    collections::HashMap,
    fs::File,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use hex::FromHex;
use pallas::{
    crypto::key::ed25519::SecretKey,
    ledger::addresses::{Network, PaymentKeyHash},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::model::{
    hydra::{
        hydra_message::HydraData,
        hydra_socket::{self, HydraSocket},
        messages::new_tx::NewTx,
    },
    tx_builder::TxBuilder,
};

use crate::{
    model::{
        game::contract::validator::Validator,
        hydra::utxo::{Datum, UTxO},
    },
    NodeConfig,
};

use super::crd::HydraDoomNode;

#[derive(Clone, Serialize, Debug)]
pub struct NodeClient {
    pub resource: Arc<HydraDoomNode>,

    #[serde(skip)]
    pub local_connection: ConnectionInfo,
    #[serde(skip)]
    pub remote_connection: ConnectionInfo,

    #[serde(skip)]
    pub tx_builder: TxBuilder,
}

#[derive(Clone, Serialize, Debug)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u32,
    pub secure: bool,
}

#[derive(Serialize)]
pub struct NodeSummary(pub NodeClient);

#[derive(Clone, Debug)]
pub struct StateUpdate {
    pub player: String,
    pub bytes: u64,
    pub kills: u64,
    pub items: u64,
    pub secrets: u64,
    pub time: Vec<u128>,
}

#[derive(Serialize, Deserialize)]
pub struct KeyEnvelope {
    #[serde(rename = "type")]
    envelope_type: String,
    description: String,
    #[serde(rename = "cborHex")]
    cbor_hex: String,
}

impl TryInto<SecretKey> for KeyEnvelope {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<SecretKey, Self::Error> {
        Ok(<[u8; 32]>::from_hex(&self.cbor_hex[4..])?.into())
    }
}

impl NodeClient {
    pub fn new(resource: Arc<HydraDoomNode>, admin_key: SecretKey) -> Result<Self> {
        let status = resource.status.as_ref().ok_or(anyhow!("no status found"))?;

        let (local_connection, remote_connection) = ConnectionInfo::from_resource(status)?;

        let node = Self {
            resource,
            local_connection,
            remote_connection,
            tx_builder: TxBuilder::new(admin_key),
        };

        Ok(node)
    }

    pub async fn new_game(&self, player_key: PaymentKeyHash) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;
        let new_game_tx = self
            .tx_builder
            .build_new_game(player_key, utxos, Network::Testnet)?; // TODO: pass in network
        let tx_hash = new_game_tx.tx_hash.0.to_vec();

        let newtx = NewTx::new(new_game_tx)?;

        hydra_socket::submit_tx_roundtrip(
            self.remote_connection.to_websocket_url().as_str(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(10),
        )
        .await?;

        Ok(tx_hash)
    }

    //TODO: don't hardcode network
    pub async fn add_player(&self, player_key: PaymentKeyHash) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;
        let add_player_tx = self
            .tx_builder
            .add_player(player_key, utxos, Network::Testnet)?;

        let tx_hash = add_player_tx.tx_hash.0.to_vec();

        let newtx = NewTx::new(add_player_tx)?;

        hydra_socket::submit_tx_roundtrip(
            self.remote_connection.to_websocket_url().as_str(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(10),
        )
        .await?;

        Ok(tx_hash)
    }

    pub async fn fetch_utxos(&self) -> Result<Vec<UTxO>> {
        let request_url = self.local_connection.to_http_url() + "/snapshot/utxo";
        let response = reqwest::get(&request_url).await.context("http error")?;

        let body = response
            .json::<HashMap<String, Value>>()
            .await
            .context("http error")?;

        let utxos = body
            .iter()
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>>>()?;

        Ok(utxos)
    }
}

impl ConnectionInfo {
    fn from_resource(resource: &super::crd::HydraDoomNodeStatus) -> Result<(Self, Self)> {
        Ok((
            ConnectionInfo::from_url(&resource.local_url)?,
            ConnectionInfo::from_url(&resource.external_url)?,
        ))
    }

    fn from_url(value: &str) -> Result<Self> {
        // default to secure connection if no schema provided
        let url = Url::parse(value)?;

        Ok(ConnectionInfo {
            host: url.host_str().context("expected a host")?.to_string(),
            port: url.port().unwrap_or(80) as u32,
            secure: url.scheme() == "https" || url.scheme() == "wss",
        })
    }

    pub fn to_websocket_url(&self) -> String {
        let schema = if self.secure { "wss" } else { "ws" };
        format!("{}://{}:{}?history=no", schema, self.host, self.port)
    }

    pub fn to_http_url(&self) -> String {
        let schema = if self.secure { "https" } else { "http" };
        format!("{}://{}:{}", schema, self.host, self.port)
    }

    pub fn to_authority(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
