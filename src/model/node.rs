use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
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
        addresses::Address,
        primitives::conway::{PlutusData, PseudoDatumOption},
        traverse::MultiEraTx,
    },
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, warn};

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
use crate::{model::hydra::utxo::UTxO, NodeConfig, SCRIPT_ADDRESS};

#[derive(Clone, Serialize)]
pub struct Node {
    #[serde(rename = "id")]
    pub head_id: Option<String>,
    #[serde(rename = "total")]
    pub stats: NodeStats,
    pub stats_file: Option<String>,
    pub region: String,
    pub max_players: usize,
    pub persisted: bool,
    pub reserved: bool,
    pub online: Arc<AtomicBool>,

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

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct LeaderboardEntry(String, u64);

impl PartialOrd for LeaderboardEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LeaderboardEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeStats {
    #[serde(default)]
    pub online_nodes: usize,
    #[serde(default)]
    pub offline_nodes: usize,
    pub total_games: u64,
    pub active_games: usize,
    pub transactions: u64,
    pub bytes: u64,

    pub kills: HashMap<String, u64>,
    pub total_kills: u64,
    pub kills_leaderboard: Vec<LeaderboardEntry>,
    pub items: HashMap<String, u64>,
    pub total_items: u64,
    pub items_leaderboard: Vec<LeaderboardEntry>,
    pub secrets: HashMap<String, u64>,
    pub total_secrets: u64,
    pub secrets_leaderboard: Vec<LeaderboardEntry>,

    pub player_play_time: HashMap<String, Vec<u128>>,
    pub total_play_time: u128,

    #[serde(skip)]
    pub pending_transactions: HashMap<Vec<u8>, StateUpdate>,
}

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

        let mut stats = if config.stats_file.is_some()
            && Path::new(config.stats_file.as_ref().unwrap()).exists()
        {
            let contents = fs::read_to_string(config.stats_file.as_ref().unwrap())
                .context("expected a stats file")?;
            serde_json::from_str(contents.as_str())?
        } else {
            NodeStats::new()
        };
        // Collapse any previous players into total time
        for (_, kills) in stats.kills.drain() {
            stats.total_kills += if kills < 10000 { kills } else { 0 };
        }
        for (_, items) in stats.items.drain() {
            stats.total_items += if items < 10000 { items } else { 0 };
        }
        for (_, secrets) in stats.secrets.drain() {
            stats.total_secrets += if secrets < 10000 { secrets } else { 0 };
        }
        for (_, play_times) in stats.player_play_time.drain() {
            stats.total_play_time += play_times.iter().sum::<u128>();
        }
        // Remove any buggy top scores
        for leaderboard in &mut [
            &mut stats.kills_leaderboard,
            &mut stats.items_leaderboard,
            &mut stats.secrets_leaderboard,
        ] {
            leaderboard.retain(|entry| entry.1 < 10000);
        }

        let socket = HydraSocket::new(
            local_connection.to_websocket_url().as_str(),
            local_connection.to_authority(),
            writer,
        );
        let node = Node {
            head_id: None,
            local_connection,
            remote_connection,
            stats,
            stats_file: config.stats_file.clone(),
            region: config.region.clone(),
            max_players: config.max_players,
            persisted: config.persisted,
            reserved: config.reserved,
            online: socket.online.clone(),

            players: Vec::new(),
            socket,
            tx_builder: TxBuilder::new(admin_key.try_into()?),
        };

        node.start_listen();
        Ok(node)
    }

    pub async fn add_player(
        &mut self,
        player: Player,
        collateral_addr: Address,
    ) -> Result<(String, String)> {
        let expired_utxos = self.cleanup_players();
        let utxos = self.fetch_utxos().await.context("Failed to fetch utxos")?;

        let (new_game_tx, player_utxo_datum) =
            self.tx_builder
                .build_new_game_state(&player, utxos, expired_utxos, collateral_addr)?;
        let player_utxo = hex::encode(new_game_tx.tx_hash.0) + "#0";

        let message: String = NewTx::new(new_game_tx)?.into();

        self.stats.total_games += 1;
        self.players.push(player);
        self.send(message).await?;

        Ok((player_utxo, hex::encode(player_utxo_datum)))
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

        self.stats
            .pending_transactions
            .insert(transaction.tx_id, state_update);

        Ok(())
    }

    pub fn cleanup_players(&mut self) -> Vec<UTxO> {
        let mut to_remove = vec![];
        for (index, player) in self.players.iter().enumerate() {
            if player.is_expired(Duration::from_secs(30)) {
                let key = hex::encode(&player.pkh);
                self.stats.total_kills += self.stats.kills.remove(&key).unwrap_or(0);
                self.stats.total_items += self.stats.items.remove(&key).unwrap_or(0);
                self.stats.total_secrets += self.stats.secrets.remove(&key).unwrap_or(0);
                self.stats.total_play_time += self
                    .stats
                    .player_play_time
                    .remove(&key)
                    .unwrap_or_default()
                    .iter()
                    .sum::<u128>();
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
impl NodeStats {
    pub fn new() -> NodeStats {
        NodeStats {
            online_nodes: 0,
            offline_nodes: 0,
            total_games: 0,
            active_games: 0,
            transactions: 0,
            bytes: 0,

            kills: HashMap::new(),
            total_kills: 0,
            kills_leaderboard: vec![],
            items: HashMap::new(),
            total_items: 0,
            items_leaderboard: vec![],
            secrets: HashMap::new(),
            total_secrets: 0,
            secrets_leaderboard: vec![],
            player_play_time: HashMap::new(),
            total_play_time: 0,

            pending_transactions: HashMap::new(),
        }
    }

    pub fn calculate_stats(&mut self, confirmed_txs: Vec<Vec<u8>>, stats_file: Option<String>) {
        for tx_id in confirmed_txs {
            match self.pending_transactions.remove(&tx_id) {
                Some(state_change) => self.update_stats(state_change),

                None => debug!(
                    "Transaction in snapshot not found in stored transactions: {:?}",
                    tx_id
                ),
            }
        }
        if let Some(stats_file) = stats_file {
            let contents = match serde_json::to_string(&self) {
                Ok(contents) => contents,
                Err(e) => {
                    warn!("failed to serialize stats {}", e);
                    return;
                }
            };
            if let Some(path) = Path::new(&stats_file).parent() {
                match fs::create_dir_all(path) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("failed to create stats directory {}", e)
                    }
                }
            }
            match fs::write(stats_file, contents).context("failed to save stats") {
                Ok(_) => {}
                Err(e) => {
                    warn!("failed to save stats file {}", e);
                }
            };
        }
    }

    fn update_stats(&mut self, state_change: StateUpdate) {
        self.transactions += 1;
        self.bytes += state_change.bytes;
        let kills = self
            .kills
            .entry(state_change.player.clone())
            .and_modify(|k| {
                *k += if state_change.kills > 10000 {
                    0
                } else {
                    state_change.kills
                }
            })
            .or_insert(state_change.kills);
        let mut min = *kills;
        let mut found = false;
        for entry in self.kills_leaderboard.iter_mut() {
            if entry.0 == state_change.player {
                if entry.1 < *kills {
                    entry.1 = *kills;
                }
                found = true;
                break;
            }
            if entry.1 < min {
                min = entry.1;
            }
        }
        if !found && (*kills > min || self.kills_leaderboard.len() < 10) {
            self.kills_leaderboard
                .push(LeaderboardEntry(state_change.player.clone(), *kills));
        }
        self.kills_leaderboard.sort();
        self.kills_leaderboard.reverse();
        self.kills_leaderboard.truncate(10);
        let items = self
            .items
            .entry(state_change.player.clone())
            .and_modify(|k| {
                *k += if state_change.items > 10000 {
                    0
                } else {
                    state_change.items
                }
            })
            .or_insert(state_change.items);
        let mut min = *items;
        let mut found = false;
        for entry in self.items_leaderboard.iter_mut() {
            if entry.0 == state_change.player {
                if entry.1 < *items {
                    entry.1 = *items;
                }
                found = true;
                break;
            }
            if entry.1 < min {
                min = entry.1;
            }
        }
        if !found && (*items > min || self.items_leaderboard.len() < 10) {
            self.items_leaderboard
                .push(LeaderboardEntry(state_change.player.clone(), *items));
        }
        self.items_leaderboard.sort();
        self.items_leaderboard.reverse();
        self.items_leaderboard.truncate(10);
        let secrets = self
            .secrets
            .entry(state_change.player.clone())
            .and_modify(|k| {
                *k += if state_change.secrets > 10000 {
                    0
                } else {
                    state_change.secrets
                }
            })
            .or_insert(state_change.secrets);
        let mut min = *secrets;
        let mut found = false;
        for entry in self.secrets_leaderboard.iter_mut() {
            if entry.0 == state_change.player {
                if entry.1 < *secrets {
                    entry.1 = *secrets;
                }
                found = true;
                break;
            }
            if entry.1 < min {
                min = entry.1;
            }
        }
        if !found && (*secrets > min || self.secrets_leaderboard.len() < 10) {
            self.secrets_leaderboard
                .push(LeaderboardEntry(state_change.player.clone(), *secrets));
        }
        self.secrets_leaderboard.sort();
        self.secrets_leaderboard.reverse();
        self.secrets_leaderboard.truncate(10);

        self.player_play_time
            .entry(state_change.player)
            .and_modify(|k| *k = state_change.time.clone())
            .or_insert(state_change.time);
    }

    pub fn join(&self, other: NodeStats, active_games: usize) -> NodeStats {
        let mut pending_transactions = self.pending_transactions.clone();
        pending_transactions.extend(other.pending_transactions);

        let mut kills = self.kills.clone();
        kills.extend(other.kills);
        let mut items = self.items.clone();
        items.extend(other.items);
        let mut secrets = self.secrets.clone();
        secrets.extend(other.secrets);

        let mut play_time = self.player_play_time.clone();
        play_time.extend(other.player_play_time);

        NodeStats {
            online_nodes: self.online_nodes + other.online_nodes,
            offline_nodes: self.offline_nodes + other.offline_nodes,
            total_games: self.total_games + other.total_games,
            active_games: self.active_games + active_games, // TODO: this is awkward; but best way to prune expired games
            transactions: self.transactions + other.transactions,
            bytes: self.bytes + other.bytes,

            kills,
            total_kills: self.total_kills + other.total_kills,
            kills_leaderboard: Self::merge_leaderboards(
                &self.kills_leaderboard,
                &other.kills_leaderboard,
            ),
            items,
            total_items: self.total_items + other.total_items,
            items_leaderboard: Self::merge_leaderboards(
                &self.items_leaderboard,
                &other.items_leaderboard,
            ),
            secrets,
            total_secrets: self.total_secrets + other.total_secrets,
            secrets_leaderboard: Self::merge_leaderboards(
                &self.secrets_leaderboard,
                &other.secrets_leaderboard,
            ),
            player_play_time: play_time,
            total_play_time: self.total_play_time + other.total_play_time,

            pending_transactions: HashMap::new(),
        }
    }

    pub fn merge_leaderboards(
        left: &[LeaderboardEntry],
        right: &[LeaderboardEntry],
    ) -> Vec<LeaderboardEntry> {
        let mut merged = vec![];

        merged.extend(left.iter().cloned());
        merged.extend(right.iter().cloned());

        merged.sort();
        merged.reverse();
        merged.truncate(10);

        merged
    }
}
