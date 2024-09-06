use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{anyhow, Result};
use async_tungstenite::{
    stream::Stream,
    tokio::{connect_async, TokioAdapter},
    tungstenite::Message,
    WebSocketStream,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc::UnboundedSender, Mutex},
    task::yield_now,
};
use tokio_native_tls::TlsStream;
use tracing::{debug, warn};

use super::hydra_message::{HydraData, HydraMessage};

#[allow(dead_code)]
#[derive(Clone)]
pub struct HydraSocket {
    url: String,
    identifier: String,
    pub online: Arc<AtomicBool>,
    writer: UnboundedSender<HydraData>,
    sender: Arc<Mutex<Option<HydraSender>>>,

    suppress_noise: bool,
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
    pub fn new(url: &str, identifier: String, writer: &UnboundedSender<HydraData>) -> Self {
        HydraSocket {
            url: url.to_string(),
            identifier,
            online: Arc::new(AtomicBool::new(false)),
            writer: writer.clone(),
            sender: Arc::new(Mutex::new(None)),

            suppress_noise: false,
        }
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
        let mut socket = self.clone();
        tokio::spawn(async move {
            socket.suppress_noise = false;
            loop {
                match socket.connect_and_listen().await {
                    Ok(()) => {
                        if !socket.suppress_noise {
                            socket.suppress_noise = true;
                            warn!("Disconnected from {}, reconnecting", socket.url);
                        }
                    }
                    Err(e) => {
                        if !socket.suppress_noise {
                            socket.suppress_noise = true;
                            warn!("Error connecting to {}: {}", socket.url, e);
                        }
                    }
                }
                socket.online.store(false, Ordering::SeqCst);
                yield_now().await;
            }
        });
    }
    async fn connect_and_listen(&mut self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        println!("Succesfully connected to {}", &self.url);
        self.suppress_noise = false;
        self.online.store(true, Ordering::SeqCst);
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
                    let message = event;

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
                self.sender.send(Message::Text(data)).await?;
                debug!("Sent message");
                Ok(())
            }
            _ => Err(anyhow!("Can only send data of variant Send")),
        }
    }
}
