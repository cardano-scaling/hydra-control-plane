use std::collections::HashMap;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use tokio::sync::mpsc::UnboundedSender;

use super::{
    hydra::{hydra_message::HydraData, hydra_socket::HydraSocket, messages::tx_valid::TxValid},
    player::Player,
};

#[derive(Clone)]
pub struct Node {
    pub uri: String,
    pub head_id: Option<String>,
    pub socket: HydraSocket,
    pub players: Vec<Player>,
    pub stats: NodeStats,
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

impl Node {
    pub async fn try_new(
        uri: &str,
        writer: &UnboundedSender<HydraData>,
        persisted: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = HydraSocket::new(uri, writer).await?;
        let node = Node {
            uri: uri.to_owned(),
            head_id: None,
            players: Vec::new(),
            socket,
            stats: NodeStats::new(persisted),
        };

        node.listen();
        Ok(node)
    }

    pub fn listen(&self) {
        let receiver = self.socket.receiver.clone();
        let uri = self.uri.clone();
        tokio::spawn(async move { receiver.lock().await.listen(uri.as_str()).await });
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
        s.end()
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
    pub fn add_transaction(&mut self, tx_id: Vec<u8>, state_change: StateUpdate) {
        self.pending_transactions.insert(tx_id, state_change);
    }

    pub fn calculate_stats(&mut self, confirmed_txs: Vec<Vec<u8>>) {
        for tx_id in confirmed_txs {
            match self.pending_transactions.remove(&tx_id) {
                Some(state_change) => self.update_stats(state_change),

                None => println!(
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

impl From<TxValid> for StateUpdate {
    fn from(value: TxValid) -> Self {
        // TODO: implement this from reading datum
        StateUpdate {
            bytes: value.cbor.len() as u64,
            kills: 0,
            items: 0,
            secrets: 0,
            play_time: 0,
        }
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
