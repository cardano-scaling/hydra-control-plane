use std::{
    collections::HashMap,
    fs::File,
    sync::{atomic::AtomicBool, Arc},
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

use super::{
    hydra::{hydra_message::HydraData, hydra_socket::HydraSocket, messages::new_tx::NewTx},
    tx_builder::TxBuilder,
};

use crate::{
    model::{game::contract::validator::Validator, hydra::utxo::UTxO},
    NodeConfig,
};

#[derive(Clone, Serialize)]
pub struct Node {
    #[serde(rename = "id")]
    pub head_id: Option<String>,
    pub region: String,
    pub max_players: usize,
    pub persisted: bool,
    pub reserved: bool,
    pub online: Arc<AtomicBool>,
    pub occupied: bool,

    #[serde(skip)]
    pub local_connection: ConnectionInfo,
    #[serde(skip)]
    pub remote_connection: ConnectionInfo,
    #[serde(skip)]
    pub socket: HydraSocket,
    #[serde(skip)]
    pub tx_builder: TxBuilder,
}

#[derive(Clone, Serialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u32,
    pub secure: bool,
}

#[derive(Serialize)]
pub struct NodeSummary(pub Node);

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

impl Node {
    pub async fn try_new(config: &NodeConfig, writer: &UnboundedSender<HydraData>) -> Result<Self> {
        let (local_connection, remote_connection) = ConnectionInfo::from_config(config)?;

        let admin_key: KeyEnvelope = serde_json::from_reader(
            File::open(&config.admin_key_file).context("unable to open key file")?,
        )
        .context("unable to parse key file")?;

        let socket = HydraSocket::new(
            local_connection.to_websocket_url().as_str(),
            local_connection.to_authority(),
            writer,
        );
        let node = Node {
            head_id: None,
            local_connection,
            remote_connection,
            region: config.region.clone(),
            max_players: config.max_players,
            persisted: config.persisted,
            reserved: config.reserved,
            online: socket.online.clone(),
            occupied: false,

            socket,
            tx_builder: TxBuilder::new(admin_key.try_into()?),
        };

        node.start_listen();
        Ok(node)
    }

    pub async fn new_game(&self, player_key: PaymentKeyHash) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;
        let new_game_tx = self
            .tx_builder
            .build_new_game(player_key, utxos, Network::Testnet)?; // TODO: pass in network
        let tx_hash = new_game_tx.tx_hash.0.to_vec();

        let message = NewTx::new(new_game_tx)?.into();
        self.send(message).await?;

        Ok(tx_hash)
    }

    //TODO: don't hardcode network
    pub async fn add_player(&self, player_key: PaymentKeyHash) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;
        let game_state_utxo = utxos
            .iter()
            .find(|utxo| utxo.address == Validator::address(Network::Testnet))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;
        let add_player_tx =
            self.tx_builder
                .add_player(player_key, game_state_utxo.clone(), Network::Testnet)?;

        let tx_hash = add_player_tx.tx_hash.0.to_vec();

        let message = NewTx::new(add_player_tx)?.into();
        self.send(message).await?;

        Ok(tx_hash)
    }

    pub fn start_listen(&self) {
        let socket = self.socket.clone();
        tokio::spawn(async move { socket.listen() });
    }

    pub async fn send(&self, message: String) -> Result<()> {
        self.socket.send(message).await
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
    fn from_config(value: &NodeConfig) -> Result<(Self, Self)> {
        Ok((
            ConnectionInfo::from_url(&value.local_url, value.port)?,
            ConnectionInfo::from_url(
                value.remote_url.as_ref().unwrap_or(&value.local_url),
                value.port,
            )?,
        ))
    }

    fn from_url(value: &str, port: u32) -> Result<Self> {
        // default to secure connection if no schema provided
        let url = Url::parse(value)?;

        Ok(ConnectionInfo {
            host: url.host_str().context("expected a host")?.to_string(),
            port,
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
