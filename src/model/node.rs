use crate::{model::hydra::utxo::UTxO, NodeConfig, SCRIPT_ADDRESS};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use hex::{FromHex, ToHex};
use pallas::{
    codec::{minicbor::decode, utils::KeepRaw},
    crypto::key::ed25519::SecretKey,
    ledger::{
        addresses::Address,
        primitives::{
            babbage::{PseudoScript, PseudoTransactionOutput},
            conway::{
                NativeScript, PlutusData, PseudoDatumOption, PseudoPostAlonzoTransactionOutput,
            },
        },
        traverse::MultiEraTx,
    },
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs::File, time::Duration};
use tokio::{sync::mpsc::UnboundedSender, time::sleep};

use super::{
    game_state::GameState,
    hydra::{
        hydra_message::HydraData,
        hydra_socket::HydraSocket,
        messages::{new_tx::NewTx, tx_valid::TxValid},
    },
    player::Player,
    tx_builder::TxBuilder,
};

#[derive(Clone, Serialize)]
pub struct Node {
    #[serde(rename = "id")]
    pub head_id: Option<String>,
    #[serde(rename = "total")]
    pub stats: NodeStats,
    pub max_players: usize,
    pub persisted: bool,

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

#[derive(Serialize, Clone)]
pub struct NodeStats {
    pub games: u64,
    pub transactions: u64,
    pub bytes: u64,
    pub kills: u64,
    pub items: u64,
    pub secrets: u64,
    pub play_time: u64,

    #[serde(skip)]
    pub pending_transactions: HashMap<Vec<u8>, StateUpdate>,
}

#[derive(Clone)]
pub struct StateUpdate {
    pub bytes: u64,
    pub kills: u64,
    pub items: u64,
    pub secrets: u64,
    pub play_time: u64,
}

#[derive(Serialize, Deserialize)]
struct KeyEnvelope {
    #[serde(rename = "type")]
    envelope_type: String,
    description: String,
    #[serde(rename = "cborHex")]
    cbor_hex: String,
}

impl TryInto<SecretKey> for KeyEnvelope {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<SecretKey, Self::Error> {
        Ok(<[u8; 32]>::from_hex(self.cbor_hex[4..].to_string())?.into())
    }
}

impl Node {
    pub async fn try_new(config: &NodeConfig, writer: &UnboundedSender<HydraData>) -> Result<Self> {
        let local_url = config.local_url.clone();
        let local_connection: ConnectionInfo = local_url.clone().try_into()?;
        let remote_connection: ConnectionInfo = config
            .remote_url
            .as_ref()
            .unwrap_or(&local_url)
            .clone()
            .try_into()?;

        let admin_key: KeyEnvelope = serde_json::from_reader(
            File::open(&config.admin_key_file).context("unable to open key file")?,
        )
        .context("unable to parse key file")?;

        let socket = HydraSocket::new(local_connection.to_websocket_url().as_str(), writer).await?;
        let mut node = Node {
            head_id: None,
            local_connection,
            remote_connection,
            stats: NodeStats::new(),
            max_players: config.max_players,
            persisted: config.persisted,
            players: Vec::new(),
            socket,
            tx_builder: TxBuilder::new(admin_key.try_into()?),
        };

        node.listen();
        Node::set_script_ref(&mut node).await?;
        Ok(node)
    }

    // This sucks and is hacky. Definitely a better way to do this, but I can't think
    async fn set_script_ref(node: &mut Node) -> Result<()> {
        let utxos = node.fetch_utxos().await.context("Failed to fetch UTxOs")?;
        let maybe_script_ref = TxBuilder::find_script_ref(utxos.clone());
        match maybe_script_ref {
            Some(script_ref) => {
                node.tx_builder.set_script_ref(&script_ref);
                debug!("Set script ref! {:?}", script_ref);
                Ok(())
            }
            None => {
                let tx = node.tx_builder.create_script_ref(utxos)?;
                let message: String = NewTx::new(tx)?.into();
                node.send(message);
                sleep(Duration::from_millis(250)).await;
                Box::pin(Node::set_script_ref(node)).await
            }
        }
    }

    pub async fn add_player(&mut self, player: Player) -> Result<String> {
        let expired_utxos = self.cleanup_players();
        let utxos = self.fetch_utxos().await.context("Failed to fetch utxos")?;

        if self.players.len() >= self.max_players {
            bail!("Max players reached");
        }

        let new_game_tx = self
            .tx_builder
            .build_new_game_state(&player, utxos, expired_utxos)?;
        let player_utxo = hex::encode(new_game_tx.tx_hash.0) + "#0";

        let message: String = NewTx::new(new_game_tx)?.into();

        self.stats.games += 1;
        self.players.push(player);
        self.send(message);

        Ok(player_utxo)
    }

    pub fn listen(&self) {
        let receiver = self.socket.receiver.clone();
        let identifier = self.local_connection.to_authority();
        tokio::spawn(async move { receiver.lock().await.listen(identifier.as_str()).await });
    }

    pub fn send(&self, message: String) {
        let sender = self.socket.sender.clone();
        tokio::spawn(async move {
            let _ = sender.lock().await.send(HydraData::Send(message)).await;
        });
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

        let tx = tx.as_babbage().context("Invalid babbage era tx")?;

        let outputs = &tx.transaction_body.outputs;
        let script_outputs = outputs
            .into_iter()
            .filter(|output| match output {
                PseudoTransactionOutput::PostAlonzo(output) => {
                    let bytes: Vec<u8> = output.address.clone().into();
                    let address = match Address::from_bytes(bytes.as_slice()) {
                        Ok(address) => address,
                        Err(_) => return false,
                    };
                    // unwrapping here because it came from hydra, so it is valid
                    let address = address.to_bech32().unwrap();

                    address.as_str() == SCRIPT_ADDRESS
                }
                _ => false,
            })
            .collect::<Vec<
                &PseudoTransactionOutput<
                    PseudoPostAlonzoTransactionOutput<
                        PseudoDatumOption<KeepRaw<PlutusData>>,
                        PseudoScript<KeepRaw<NativeScript>>,
                    >,
                >,
            >>();

        if script_outputs.len() != 1 {
            bail!("Invalid number of script outputs");
        }

        let script_output = script_outputs.first().unwrap();
        match script_output {
            PseudoTransactionOutput::PostAlonzo(output) => {
                let datum = match output.datum_option.as_ref() {
                    Some(PseudoDatumOption::Data(datum)) => datum,
                    // If there's no datum, or a datum hash, it's an unrelated transaction
                    _ => return Ok(()),
                }
                .0
                .raw_cbor();

                let data = match decode::<PlutusData>(datum) {
                    Ok(data) => data,
                    Err(_) => bail!("Failed to deserialize datum"),
                };

                let game_state: GameState = data.try_into()?;

                let player = match self
                    .players
                    .iter_mut()
                    .find(|player| player.pkh == game_state.owner)
                {
                    Some(player) => player,
                    None => {
                        warn!(
                            "Player not found {}",
                            game_state.owner.encode_hex::<String>()
                        );
                        return Ok(());
                    }
                };

                // TODO: actually find the index
                let utxo =
                    UTxO::try_from_pallas(hex::encode(&transaction.tx_id).as_str(), 0, output)?;
                let timestamp: u64 =
                    transaction.timestamp.parse::<DateTime<Utc>>()?.timestamp() as u64;
                player.utxo = Some(utxo);
                player.utxo_time = timestamp;

                let state_update =
                    player.generate_state_update(transaction.cbor.len() as u64, game_state);

                self.stats
                    .pending_transactions
                    .insert(transaction.tx_id, state_update);

                Ok(())
            }
            _ => bail!("Invalid output type"),
        }
    }

    pub fn cleanup_players(&mut self) -> Vec<UTxO> {
        let mut to_remove = vec![];
        for (index, player) in self.players.iter().enumerate() {
            if player.is_expired(Duration::from_secs(60 * 5)) {
                println!("Player expired: {:?}", hex::encode(&player.pkh));
                to_remove.push(index);
            }
        }

        let mut utxos = vec![];
        for index in to_remove.iter().rev() {
            println!(
                "Removing player: {:?}",
                hex::encode(&self.players[*index].pkh)
            );
            if let Some(utxo) = self.players.remove(*index).utxo {
                utxos.push(utxo);
            }
        }

        utxos
    }
}

impl TryFrom<String> for ConnectionInfo {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        // default to secure connection if no schema provided
        match parts.len() {
            2 => {
                let host = parts[0].to_string();
                let port = parts[1].parse::<u32>()?;

                Ok(ConnectionInfo {
                    host,
                    port,
                    secure: true,
                })
            }
            3 => {
                let schema = parts[0].to_string();
                let port = parts[2].parse::<u32>()?;
                let host = parts[1]
                    .to_string()
                    .split("//")
                    .last()
                    .context("Invalid host")?
                    .to_string();

                let secure = schema == "https" || schema == "wss";
                Ok(ConnectionInfo { host, port, secure })
            }
            _ => {
                bail!("Invalid uri");
            }
        }
    }
}

impl ConnectionInfo {
    pub fn to_websocket_url(&self) -> String {
        let schema = if self.secure { "wss" } else { "ws" };
        format!("{}://{}:{}", schema, self.host, self.port)
    }

    pub fn to_http_url(&self) -> String {
        let schema = if self.secure { "https" } else { "http" };
        format!("{}://{}:{}", schema, self.host, self.port)
    }

    pub fn to_authority(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
impl NodeStats {
    pub fn new() -> NodeStats {
        NodeStats {
            games: 0,
            transactions: 0,
            bytes: 0,
            kills: 0,
            items: 0,
            secrets: 0,
            play_time: 0,
            pending_transactions: HashMap::new(),
        }
    }

    pub fn calculate_stats(&mut self, confirmed_txs: Vec<Vec<u8>>) {
        for tx_id in confirmed_txs {
            match self.pending_transactions.remove(&tx_id) {
                Some(state_change) => self.update_stats(state_change),

                None => debug!(
                    "Transaction in snapshot not found in stored transactions: {:?}",
                    tx_id
                ),
            }
        }
    }

    fn update_stats(&mut self, state_change: StateUpdate) {
        self.transactions += 1;
        self.bytes += state_change.bytes;
        self.kills += state_change.kills;
        self.items += state_change.items;
        self.secrets += state_change.secrets;
        self.play_time += state_change.play_time;
    }

    pub fn join(&self, other: NodeStats) -> NodeStats {
        let mut pending_transactions = self.pending_transactions.clone();
        pending_transactions.extend(other.pending_transactions);

        NodeStats {
            games: self.games + other.games,
            transactions: self.transactions + other.transactions,
            bytes: self.bytes + other.bytes,
            kills: self.kills + other.kills,
            items: self.items + other.items,
            secrets: self.secrets + other.secrets,
            play_time: self.play_time + other.play_time,
            pending_transactions,
        }
    }
}
