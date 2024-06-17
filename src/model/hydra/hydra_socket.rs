use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use super::hydra_message::HydraMessage;

pub struct HydraSocket {
    pub receiver: Arc<Mutex<HydraReceiver>>,
    sender: Arc<Mutex<HydraSender>>,
    pub connected: bool,
}

pub struct HydraReceiver {
    receiver: SplitStream<WebSocketStream<TokioAdapter<TcpStream>>>,
    writer: UnboundedSender<String>,
}

pub struct HydraSender {
    sender: SplitSink<WebSocketStream<TokioAdapter<TcpStream>>, Message>,
}

impl HydraSocket {
    pub async fn new(
        url: &str,
        writer: &UnboundedSender<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(url).await?;
        let (sender, receiver) = ws_stream.split();

        Ok(HydraSocket {
            receiver: Arc::new(Mutex::new(HydraReceiver {
                receiver,
                writer: writer.clone(),
            })),
            sender: Arc::new(Mutex::new(HydraSender { sender })),
            connected: true,
        })
    }
}

impl HydraReceiver {
    pub async fn listen(&mut self) {
        while let Some(msg) = self.receiver.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    println!("Error receiving message: {:?}", e);
                    continue;
                }
            };

            match HydraMessage::try_from(msg) {
                Ok(hydra_message) => match hydra_message {
                    HydraMessage::Ping(payload) => {
                        println!("Received ping: {:?}", payload);
                    }

                    HydraMessage::HydraEvent(event) => {
                        println!("Received event: {:?}", event);
                    }
                },
                Err(e) => {
                    println!("Error parsing message: {:?}", e);
                }
            }
        }
    }
}
