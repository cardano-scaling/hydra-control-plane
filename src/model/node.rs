use serde::ser::{Serialize, SerializeStruct, Serializer};
use tokio::sync::mpsc::UnboundedSender;

use super::{
    hydra::{hydra_message::HydraData, hydra_socket::HydraSocket},
    player::Player,
};

#[derive(Clone)]
pub struct Node {
    pub uri: String,
    pub head_id: Option<String>,
    pub socket: HydraSocket,
    pub players: Vec<Player>,
    pub transaction_count: u64,
}

impl Node {
    pub async fn try_new(
        uri: &str,
        writer: &UnboundedSender<HydraData>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = HydraSocket::new(uri, writer).await?;
        let node = Node {
            uri: uri.to_owned(),
            head_id: None,
            players: Vec::new(),
            socket,
            transaction_count: 0,
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
        s.serialize_field("uri", &self.uri)?;
        s.serialize_field("head_id", &self.head_id)?;
        s.serialize_field("players", &self.players.len())?;
        s.serialize_field("transaction_count", &self.transaction_count)?;
        s.skip_field("socket")?;
        s.end()
    }
}
