use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct HydraSocket {
    pub receiver: Arc<Mutex<HydraReceiver>>,
    sender: Arc<Mutex<HydraSender>>,
    pub connected: bool,
}

// Thinking: we should use DI to inject a struct that implements a "HydraHandler" trait
// to decouple the business logic from the communication logic.
// Also allows us to have the nod be a hydra handler, and potentially update state
// Thread safety gets weird, but feels like the intuitive way to go.
pub struct HydraReceiver {
    receiver: SplitStream<WebSocketStream<TokioAdapter<TcpStream>>>,
}

pub struct HydraSender {
    sender: SplitSink<WebSocketStream<TokioAdapter<TcpStream>>, Message>,
}

impl HydraSocket {
    pub async fn new(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(url).await?;
        let (sender, receiver) = ws_stream.split();

        Ok(HydraSocket {
            receiver: Arc::new(Mutex::new(HydraReceiver { receiver })),
            sender: Arc::new(Mutex::new(HydraSender { sender })),
            connected: true,
        })
    }
}

impl HydraReceiver {
    pub async fn listen(&mut self) {
        while let Some(msg) = self.receiver.next().await {
            match msg {
                Ok(msg) => {
                    println!("Received message: {:?}", msg);
                }
                Err(e) => {
                    eprintln!("Error receiving message: {:?}", e);
                }
            }
        }
    }
}
