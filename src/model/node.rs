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
    pub players: Vec<Player>,
    pub socket: HydraSocket,
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
        let mut s = serializer.serialize_struct("Node", 3)?;
        s.serialize_field("uri", &self.uri)?;
        s.serialize_field("head_id", &self.head_id)?;
        s.serialize_field("players", &self.players.len())?;
        s.skip_field("socket")?;
        s.end()
    }
}
