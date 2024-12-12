use anyhow::{anyhow, Context, Result};
use hex::FromHex;
use pallas::{crypto::key::ed25519::SecretKey, ledger::addresses::Network};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use tracing::debug;

use crate::model::{
    game::{
        contract::{
            game_state::{GameState, PaymentCredential},
            validator::Validator,
        },
        player::Player,
    },
    hydra::{
        hydra_socket,
        messages::{new_tx::NewTx, Transaction},
    },
    tx_builder::TxBuilder,
};

use crate::model::hydra::utxo::UTxO;

#[derive(Clone, Serialize, Debug)]
pub struct NodeClient {
    #[serde(skip)]
    pub connection: ConnectionInfo,

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

impl TryInto<Vec<u8>> for KeyEnvelope {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        Ok(<[u8; 32]>::from_hex(&self.cbor_hex[4..])?.into())
    }
}

impl NodeClient {
    pub fn new(connection: ConnectionInfo, admin_key: SecretKey, network: Network) -> Self {
        Self {
            connection,
            tx_builder: TxBuilder::new(admin_key, network),
        }
    }

    pub async fn new_game(
        &self,
        player: Option<Player>,
        player_count: u64,
        bot_count: u64,
    ) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;
        // Removing for now, to make iterative development easier
        // if utxos
        //     .iter()
        //     .any(|utxo| GameState::try_from(utxo.datum.clone()).is_ok())
        // {
        //     bail!("game UTxO already exists")
        // }

        let new_game_tx = self
            .tx_builder
            .new_game(player, utxos, player_count, bot_count)
            .context("failed to build transaction")?; // TODO: pass in network
        debug!("new game tx: {}", hex::encode(&new_game_tx.tx_bytes));

        let tx_hash = new_game_tx.tx_hash.0.to_vec();
        let newtx = NewTx::new(new_game_tx).context("failed to build new tx message")?;

        hydra_socket::submit_tx_roundtrip(
            &self.connection.to_websocket_url(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(10),
        )
        .await?;

        Ok(tx_hash)
    }

    pub async fn start_game(&self) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let start_game_tx = self
            .tx_builder
            .start_game(utxos)
            .context("failed to build transaction")?;

        debug!("start game tx: {}", hex::encode(&start_game_tx.tx_bytes));

        let tx_hash = start_game_tx.tx_hash.0.to_vec();

        let new_tx = NewTx::new(start_game_tx).context("failed to build NewTx message")?;
        hydra_socket::submit_tx_roundtrip(
            &self.connection.to_websocket_url(),
            new_tx, // TODO: make this configurable
            Duration::from_secs(30),
        )
        .await
        .context("failed to submit transaction")?;

        Ok(tx_hash)
    }

    pub async fn add_player(&self, player: Player) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;
        // This logic prevents players from joining the same game twice.
        // This is a really gross way to handle it, just doing it as a bandaid fix
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(self.tx_builder.network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let game_state = GameState::try_from(game_state_utxo.datum.clone())?;
        if game_state.players.contains(&player.signing_key.into()) {
            let outbound_player_address =
                player.outbound_address(self.tx_builder.admin_pkh, self.tx_builder.network)?;
            let player_utxo = utxos
                .clone()
                .into_iter()
                .find(|utxo| utxo.address == outbound_player_address)
                .ok_or_else(|| anyhow!("player state utxo not found"))?;

            return Ok(player_utxo.hash);
        }

        if game_state.players.len() > 2 {
            return Err(anyhow!("too many players"));
        }
        // Previous add player logic
        let add_player_tx = self
            .tx_builder
            .add_player(player, utxos)
            .context("failed to build transaction")?;

        debug!("add player tx: {}", hex::encode(&add_player_tx.tx_bytes));

        let tx_hash = add_player_tx.tx_hash.0.to_vec();

        let newtx = NewTx::new(add_player_tx).context("failed to construct newtx message")?;

        hydra_socket::submit_tx_roundtrip(
            &self.connection.to_websocket_url(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(10),
        )
        .await?;

        Ok(tx_hash)
    }

    pub async fn cleanup_game(&self) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let cleanup_tx = self
            .tx_builder
            .cleanup_game(utxos)
            .context("failed to build transaction")?;

        debug!("cleanup tx: {}", hex::encode(&cleanup_tx.tx_bytes));

        let tx_hash = cleanup_tx.tx_hash.0.to_vec();

        let newtx = NewTx::new(cleanup_tx).context("failed to construct newtx message")?;
        hydra_socket::submit_tx_roundtrip(
            &self.connection.to_websocket_url(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(10),
        )
        .await?;

        Ok(tx_hash)
    }

    // This is just used for testing for now, always aborting the game
    // TODO: actually handle winner and losers
    pub async fn end_game(&self) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let end_game_tx = self
            .tx_builder
            .end_game(None, utxos)
            .context("failed to build transaction")?;

        debug!("end_game_tx tx: {}", hex::encode(&end_game_tx.tx_bytes));

        let tx_hash = end_game_tx.tx_hash.0.to_vec();

        let newtx = NewTx::new(end_game_tx).context("failed to construct newtx message")?;
        hydra_socket::submit_tx_roundtrip(
            &self.connection.to_websocket_url(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(3),
        )
        .await?;

        Ok(tx_hash)
    }

    pub async fn fetch_utxos(&self) -> Result<Vec<UTxO>> {
        let request_url = self.connection.to_http_url() + "/snapshot/utxo";
        let response = reqwest::get(&request_url).await.context("http error")?;

        let body = response
            .json::<HashMap<String, Value>>()
            .await
            .context("http error")?;

        let utxos = body
            .iter()
            .map(|(key, value)| {
                println!("Trying to build UTxO from Value: {:?}", value);
                UTxO::try_from_value(key, value)
            })
            .collect::<Result<Vec<UTxO>>>()
            .context("failed to deserialize utxos")?;

        Ok(utxos)
    }

    pub async fn sample_txs(&self, count: usize) -> Result<Vec<Transaction>> {
        //TODO: make duration configurable
        hydra_socket::sample_txs(
            &format!("{}/?history=no", &self.connection.to_websocket_url()),
            count,
            Duration::from_secs(30),
        )
        .await
    }
}

impl ConnectionInfo {
    pub fn from_resource(resource: &super::crd::HydraDoomNodeStatus) -> Result<(Self, Self)> {
        Ok((
            ConnectionInfo::from_url(&resource.local_url)?,
            ConnectionInfo::from_url(&resource.external_url)?,
        ))
    }

    pub fn from_url(value: &str) -> Result<Self> {
        // default to secure connection if no schema provided
        let url = Url::parse(value)?;
        let host = url.host_str().context("expected a host")?.to_string();
        let secure = url.scheme() == "https" || url.scheme() == "wss";
        let port = url.port().unwrap_or(if secure { 443 } else { 80 }) as u32;

        Ok(ConnectionInfo { host, secure, port })
    }

    pub fn to_websocket_url(&self) -> String {
        let schema = if self.secure { "wss" } else { "ws" };
        format!("{}://{}:{}", schema, self.host, self.port)
    }

    pub fn to_http_url(&self) -> String {
        let schema = if self.secure { "https" } else { "http" };

        format!("{}://{}:{}", schema, self.host, self.port)
    }

    #[allow(dead_code)]
    pub fn to_authority(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
