use super::{hydra_socket::HydraSocket, player::Player};

pub struct Node {
    pub uri: String,
    pub players: Vec<Player>,
    pub socket: HydraSocket,
}

impl Node {
    pub async fn try_new(uri: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = HydraSocket::new(uri).await?;
        let node = Node {
            uri: uri.to_owned(),
            players: Vec::new(),
            socket,
        };

        node.listen();
        Ok(node)
    }

    pub fn listen(&self) {
        let receiver = self.socket.receiver.clone();
        tokio::spawn(async move { receiver.lock().await.listen().await });
    }
}
