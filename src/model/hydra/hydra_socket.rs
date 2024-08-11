use anyhow::{anyhow, Result};
use async_tungstenite::stream::Stream;
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
use tokio::task::yield_now;
use tokio_native_tls::TlsStream;

use super::hydra_message::{HydraData, HydraEventMessage, HydraMessage};

#[allow(dead_code)]
#[derive(Clone)]
pub struct HydraSocket {
    url: String,
    identifier: String,
    writer: UnboundedSender<HydraData>,
    sender: Arc<Mutex<Option<HydraSender>>>,
}

pub type HydraSource = SplitStream<
    WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
>;
pub type HydraSink = SplitSink<
    WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
    Message,
>;
pub struct HydraSender {
    sender: HydraSink,
}

impl HydraSocket {
    pub async fn new(
        url: &str,
        identifier: String,
        writer: &UnboundedSender<HydraData>,
    ) -> Result<Self> {
        Ok(HydraSocket {
            url: url.to_string(),
            identifier,
            writer: writer.clone(),
            sender: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn send(&self, message: String) -> Result<()> {
        // If the sender is None, we aren't currently connected, so spin loop until we're reconnected
        loop {
            let mut sender = self.sender.lock().await;
            if let Some(sender) = sender.as_mut() {
                return sender.send(HydraData::Send(message)).await;
            }
            // Make sure we don't kill the CPU
            yield_now().await;
        }
    }

    pub fn listen(&self) {
        let socket = self.clone();
        tokio::spawn(async move {
            loop {
                match socket.connect_and_listen().await {
                    Ok(()) => {
                        warn!("Disconnected from {}, reconnecting", socket.url);
                    }
                    Err(e) => {
                        warn!("Error connecting to {}: {}", socket.url, e);
                    }
                }
                yield_now().await;
            }
        });
    }
    async fn connect_and_listen(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (sender, receiver) = ws_stream.split();
        {
            let mut sender_lock = self.sender.lock().await;
            *sender_lock = Some(HydraSender { sender });
        }
        self.process_messages(receiver).await?;
        Ok(())
    }

    async fn process_messages(&self, mut receiver: HydraSource) -> Result<()> {
        while let Some(msg) = receiver.next().await {
            let msg = msg?;
            let hydra_message = HydraMessage::try_from(msg)?;
            match hydra_message {
                HydraMessage::Ping(payload) => {
                    debug!("Received ping: {:?}", payload);
                }

                HydraMessage::HydraEvent(event) => {
                    let message = HydraEventMessage::from(event);

                    let data = HydraData::Received {
                        authority: self.identifier.clone(),
                        message,
                    };
                    self.writer.send(data)?;
                }
            }
        }
        Ok(())
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
