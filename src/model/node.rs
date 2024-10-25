use std::{
    collections::HashMap,
    fs::File,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use hex::FromHex;
use pallas::{
    codec::minicbor::decode,
    crypto::key::ed25519::SecretKey,
    ledger::{
        addresses::{Address, Network, PaymentKeyHash},
        primitives::conway::{PlutusData, PseudoDatumOption},
        traverse::MultiEraTx,
    },
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;

use super::{
    game_state::GameState,
    hydra::{
        hydra_message::HydraData,
        hydra_socket::HydraSocket,
        messages::{init, new_tx::NewTx, tx_valid::TxValid},
    },
    player::Player,
    tx_builder::TxBuilder,
};

use crate::SCRIPT_ADDRESS;
use crate::{model::hydra::utxo::UTxO, NodeConfig};

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
    pub players: Vec<Player>,
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

            players: Vec::new(),
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

    pub fn add_transaction(&mut self, transaction: TxValid) -> Result<()> {
        let bytes = transaction.cbor.as_slice();
        let tx = MultiEraTx::decode(bytes).context("Failed to decode transaction")?;

        //let tx = tx.as_babbage().context("Invalid babbage era tx")?;

        let outputs = tx.outputs();
        let script_outputs: Vec<_> = outputs
            .iter()
            .filter(|output| (output.address().unwrap().to_bech32().unwrap() == SCRIPT_ADDRESS))
            .collect();

        if script_outputs.len() != 1 {
            bail!("Invalid number of script outputs");
        }

        let script_output = script_outputs.first().unwrap();

        let datum = match script_output.datum() {
            Some(PseudoDatumOption::Data(x)) => x.raw_cbor(),
            // If there's no datum, or a datum hash, it's an unrelated transaction
            _ => return Ok(()),
        };

        let data = match decode::<PlutusData>(datum) {
            Ok(data) => data,
            Err(_) => bail!("Failed to deserialize datum"),
        };

        let game_state_result: Result<GameState> = data.try_into();
        let game_state = game_state_result.context("invalid game state")?;

        let player = match self
            .players
            .iter_mut()
            .find(|player| player.pkh == game_state.owner)
        {
            Some(player) => player,
            None => {
                // We must have restarted, or the player was created through another control plane; create the player now
                warn!(
                    "Unrecognized player, adding: {}",
                    hex::encode(&game_state.owner)
                );
                self.players.push(Player {
                    pkh: game_state.owner.clone(),
                    utxo: None,
                    game_state: Some(game_state.clone()),
                    utxo_time: 0,
                });
                self.players
                    .iter_mut()
                    .find(|player| player.pkh == game_state.owner)
                    .expect("Just added")
            }
        };

        // TODO: actually find the index
        let utxo =
            UTxO::try_from_pallas(hex::encode(&transaction.tx_id).as_str(), 0, &script_output)
                .context("invalid utxo")?;

        let timestamp: u128 = transaction
            .timestamp
            .parse::<DateTime<Utc>>()
            .context("timestamp")?
            .timestamp() as u128;

        player.utxo = Some(utxo);
        player.utxo_time = timestamp;

        let state_update = player.generate_state_update(transaction.cbor.len() as u64, game_state);

        Ok(())
    }

    pub fn cleanup_players(&mut self) -> Vec<UTxO> {
        let mut to_remove = vec![];
        for (index, player) in self.players.iter().enumerate() {
            if player.is_expired(Duration::from_secs(30)) {
                let key = hex::encode(&player.pkh);
                to_remove.push(index);
            }
        }

        let mut utxos = vec![];
        for index in to_remove.iter().rev() {
            if let Some(utxo) = self.players.remove(*index).utxo {
                utxos.push(utxo);
            }
        }

        utxos
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
