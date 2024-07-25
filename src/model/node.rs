use crate::{model::hydra::utxo::UTxO, NodeConfig, SCRIPT_ADDRESS};
use anyhow::{bail, Context, Result};
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
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
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

#[derive(Clone)]
pub struct Node {
    pub connection_info: ConnectionInfo,
    pub head_id: Option<String>,
    pub socket: HydraSocket,
    pub players: Vec<Player>,
    pub stats: NodeStats,
    pub tx_builder: TxBuilder,
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u32,
    pub secure: bool,
}
pub struct NodeSummary(pub Node);

#[derive(Clone)]
pub struct NodeStats {
    pub persisted: bool,
    pub transactions: u64,
    pub bytes: u64,
    pub kills: u64,
    pub items: u64,
    pub secrets: u64,
    pub play_time: u64,
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
        let connection_info: ConnectionInfo = config.connection_url.to_string().try_into()?;

        let admin_key: KeyEnvelope = serde_json::from_reader(
            File::open(&config.admin_key_file).context("unable to open key file")?,
        )
        .context("unable to parse key file")?;

        let socket = HydraSocket::new(connection_info.to_websocket_url().as_str(), writer).await?;
        let mut node = Node {
            connection_info,
            head_id: None,
            players: Vec::new(),
            socket,
            stats: NodeStats::new(config.persisted),
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

    pub async fn add_player(&mut self, player: Player) -> Result<()> {
        let utxos = self.fetch_utxos().await.context("Failed to fetch utxos")?;

        let new_game_tx = self.tx_builder.build_new_game_state(&player, utxos)?;

        let message: String = NewTx::new(new_game_tx)?.into();

        self.players.push(player);
        self.send(message);

        Ok(())
    }

    pub fn listen(&self) {
        let receiver = self.socket.receiver.clone();
        let identifier = self.connection_info.to_authority();
        tokio::spawn(async move { receiver.lock().await.listen(identifier.as_str()).await });
    }

    pub fn send(&self, message: String) {
        let sender = self.socket.sender.clone();
        tokio::spawn(async move {
            let _ = sender.lock().await.send(HydraData::Send(message)).await;
        });
    }

    pub async fn fetch_utxos(&self) -> Result<Vec<UTxO>> {
        let request_url = self.connection_info.to_http_url() + "/snapshot/utxo";
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
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Node", 4)?;
        s.serialize_field("id", &self.head_id)?;
        s.serialize_field("total", &self.stats)?;
        // TODO: Make the active games count match the openapi schema
        s.serialize_field("active_games", &self.players.len())?;
        s.skip_field("socket")?;
        s.skip_field("ephemeral")?;
        s.skip_field("connection_info")?;
        s.end()
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
    pub fn new(persisted: bool) -> NodeStats {
        NodeStats {
            persisted,
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
            persisted: self.persisted && other.persisted,
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

impl Serialize for NodeStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("NodeStats", 6)?;
        s.serialize_field("transactions", &self.transactions)?;
        s.serialize_field("bytes", &self.bytes)?;
        s.serialize_field("kills", &self.kills)?;
        s.serialize_field("items", &self.items)?;
        s.serialize_field("secrets", &self.secrets)?;
        s.serialize_field("play_time", &self.play_time)?;
        s.skip_field("pending_transactions")?;
        s.end()
    }
}

impl Serialize for NodeSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("NodeSummary", 3)?;
        s.serialize_field("id", &self.0.head_id)?;
        s.serialize_field("active_games", &self.0.players.len())?;
        s.serialize_field("persisted", &self.0.stats.persisted)?;
        s.end()
    }
}
