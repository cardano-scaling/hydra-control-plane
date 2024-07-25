use anyhow::{anyhow, Result};
use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::SinkExt;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use super::hydra_message::{HydraData, HydraEventMessage, HydraMessage};

#[allow(dead_code)]
#[derive(Clone)]
pub struct HydraSocket {
    pub receiver: Arc<Mutex<HydraReceiver>>,
    pub sender: Arc<Mutex<HydraSender>>,
    pub connected: bool,
}

pub struct HydraReceiver {
    receiver: SplitStream<WebSocketStream<TokioAdapter<TcpStream>>>,
    writer: UnboundedSender<HydraData>,
}

pub struct HydraSender {
    sender: SplitSink<WebSocketStream<TokioAdapter<TcpStream>>, Message>,
}

impl HydraSocket {
    pub async fn new(url: &str, writer: &UnboundedSender<HydraData>) -> Result<Self> {
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

impl HydraSender {
    pub async fn send(&mut self, message: HydraData) -> Result<()> {
        match message {
            HydraData::Send(data) => {
                let _ = self.sender.send(Message::Text(data)).await?;
                debug!("Sent message");
                Ok(())
            }
            _ => Err(anyhow!("Can only send data of variant Send")),
        }
    }
}

impl HydraReceiver {
    pub async fn listen(&mut self, node_identifier: &str) {
        while let Some(msg) = self.receiver.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    warn!("Error receiving message: {:?}", e);
                    continue;
                }
            };

            match HydraMessage::try_from(msg) {
                Ok(hydra_message) => match hydra_message {
                    HydraMessage::Ping(payload) => {
                        debug!("Received ping: {:?}", payload);
                    }

                    HydraMessage::HydraEvent(event) => {
                        let message = HydraEventMessage::from(event);

                        let data = HydraData::Received {
                            authority: node_identifier.to_string(),
                            message,
                        };
                        let _ = self.writer.send(data);
                    }
                },
                Err(e) => {
                    warn!("Error parsing message: {:?}", e);
                }
            }
        }
    }
}
