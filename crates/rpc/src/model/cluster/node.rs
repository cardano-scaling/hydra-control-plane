use anyhow::{anyhow, Context, Result};
use hex::FromHex;
use pallas::{
    crypto::key::ed25519::SecretKey,
    ledger::addresses::{Address, Network},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::debug;

use crate::model::{
    game::player::Player,
    hydra::{
        hydra_socket,
        messages::{new_tx::NewTx, tx_valid::TxValid},
    },
    tx_builder::TxBuilder,
};

use crate::model::hydra::utxo::UTxO;

use super::crd::HydraDoomNode;

#[derive(Clone, Serialize, Debug)]
pub struct NodeClient {
    pub resource: Arc<HydraDoomNode>,

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
    pub fn new(resource: Arc<HydraDoomNode>, admin_key: SecretKey, remote: bool) -> Result<Self> {
        let status = resource.status.as_ref().ok_or(anyhow!("no status found"))?;

        let (local_connection, remote_connection) = ConnectionInfo::from_resource(status)?;

        let node = Self {
            resource,
            connection: if remote {
                remote_connection
            } else {
                local_connection
            },
            tx_builder: TxBuilder::new(admin_key),
        };

        Ok(node)
    }

    pub async fn new_game(&self, player: Player) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let new_game_tx = self
            .tx_builder
            .build_new_game(player, utxos, Network::Testnet)
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

    //TODO: don't hardcode network
    pub async fn add_player(&self, player: Player) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let add_player_tx = self
            .tx_builder
            .add_player(player, utxos, Network::Testnet)
            .context("failed to build transaction")?;

        debug!("add player tx: {}", hex::encode(&add_player_tx.tx_bytes));

        let tx_hash = add_player_tx.tx_hash.0.to_vec();

        let newtx = NewTx::new(add_player_tx).context("failed to construct newtx message")?;

        hydra_socket::submit_tx_roundtrip(
            &self.connection.to_websocket_url(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(30),
        )
        .await?;

        Ok(tx_hash)
    }

    pub async fn cleanup_game(&self) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let cleanup_tx = self
            .tx_builder
            .cleanup_game(utxos, Network::Testnet)
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

    // Just using this for testing now, hardcoding some values
    pub async fn end_game(&self) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let player = match Address::from_bech32(
            "addr_test1qpq0htjtaygzwtj3h4akj2mvzaxgpru4yje4ca9a507jtdw5pcy8kzccynfps4ayhmtc38j6tyjrkyfccdytnxwnd6psfelznq",
        )
        .expect("Failed to decode player address")
        {
            Address::Shelley(shelley) => shelley.payment().as_hash().clone(),
            _ => panic!("Expected Shelley address"),
        };

        let end_game_tx = self
            .tx_builder
            .end_game(player.into(), false, utxos, Network::Testnet)
            .context("failed to build transaction")?;

        debug!("end_game_tx tx: {}", hex::encode(&end_game_tx.tx_bytes));

        let tx_hash = end_game_tx.tx_hash.0.to_vec();

        let newtx = NewTx::new(end_game_tx).context("failed to construct newtx message")?;
        hydra_socket::submit_tx_roundtrip(
            &self.connection.to_websocket_url(),
            newtx,
            // TODO: make this configurable
            Duration::from_secs(10),
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
            .map(|(key, value)| UTxO::try_from_value(key, value))
            .collect::<Result<Vec<UTxO>>>()?;

        Ok(utxos)
    }

    pub async fn sample_txs(&self, count: usize) -> Result<Vec<TxValid>> {
        //TODO: make duration configurable
        hydra_socket::sample_txs(
            &format!("{}/?history=no", &self.connection.to_websocket_url()),
            count,
            Duration::from_secs(10),
        )
        .await
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
