use anyhow::{anyhow, bail, Context, Result};
use hex::FromHex;
use pallas::{
    crypto::key::ed25519::SecretKey,
    ledger::{
        addresses::Network,
        primitives::{alonzo, PlutusData},
        traverse::OutputRef,
    },
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, ops::Deref, time::Duration};
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
        utxo::Datum,
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

#[derive(Serialize, Deserialize, Debug)]
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

    pub async fn new_game(&self, player: Option<Player>) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;

        let new_game_tx = self
            .tx_builder
            .new_game(player, utxos)
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

    pub async fn cleanup_game(&self, series_ref: OutputRef, played_games: u64) -> Result<Vec<u8>> {
        let utxos = self.fetch_utxos().await.context("failed to fetch UTxOs")?;
        let series_utxo = utxos
            .iter()
            .find(|utxo| utxo.hash == series_ref.hash().deref() && utxo.index == series_ref.index())
            .ok_or(anyhow!("Missing series utxo"))?
            .to_owned();

        let (finished_games, players) = match series_utxo.datum {
            Datum::Inline(data) => match data {
                PlutusData::Constr(constr) => {
                    let finished_games = match constr.fields[0] {
                        PlutusData::BigInt(alonzo::BigInt::Int(int)) => u64::try_from(int.0)?,
                        _ => bail!("invalid finished games"),
                    };

                    let players: Vec<PaymentCredential> = match constr.fields[2].clone() {
                        PlutusData::Array(array) => {
                            let mut players = Vec::new();
                            for player in array.to_vec() {
                                players.push(player.try_into().context("players")?);
                            }

                            players
                        }
                        _ => bail!("invalid players"),
                    };

                    (finished_games, players)
                }
                _ => bail!("invalid datum data"),
            },
            _ => bail!("invalid datum type"),
        };

        if finished_games != played_games {
            bail!("game has not been stored yet")
        }

        let cleanup_tx = self
            .tx_builder
            .cleanup_game(utxos, players)
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
