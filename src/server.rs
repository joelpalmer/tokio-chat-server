use crate::protocol::ChatMessage;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::time::{Duration, timeout};
use tracing::{Level, debug, error, info, span};
use tracing_futures::Instrument;

pub struct ChatServer {
    listener: TcpListener,
    broadcast_tx: broadcast::Sender<String>,
}

impl ChatServer {
    pub async fn new(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let (broadcast_tx, _) = broadcast::channel(100);
        info!("Chat server bound to {}", addr);
        Ok(ChatServer {
            listener,
            broadcast_tx,
        })
    }

    pub async fn run(self) -> Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            let broadcast_tx = self.broadcast_tx.clone();
            let broadcast_rx = broadcast_tx.subscribe();
            info!("Accepted connection from {}", addr);

            tokio::spawn(
                handle_client(socket, addr, broadcast_tx, broadcast_rx)
                    .instrument(span!(Level::INFO, "handle_client", client_addr = %addr)),
            );
        }
    }
}

async fn handle_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    broadcast_tx: broadcast::Sender<String>,
    mut broadcast_rx: broadcast::Receiver<String>,
) -> Result<()> {
    info!("Handling client {}", addr);
    let mut buffer = [0; 1024];
    let read_timeout = Duration::from_secs(30);

    loop {
        tokio::select! {
            result = timeout(read_timeout, socket.read(&mut buffer)) => {
                match result {
                    Ok(Ok(0)) => {
                        info!("Client {} disconnected", addr);
                        return Ok(());
                    }
                    Ok(Ok(n)) => {
                        let raw = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        if !raw.is_empty() {
                            let _span = span!(Level::DEBUG, "process_message", message = %raw).entered();
                            let message = serde_json::from_str::<ChatMessage>(&raw)
                                .or_else(|_| ChatMessage::from_raw(&raw))?;
                            let json = message.to_json()?;
                            let formatted = format!("{}: {}", addr, json);
                            debug!("Broadcasting: {}", formatted);
                            broadcast_tx.send(formatted)?;
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Read error for {}: {:?}", addr, e);
                        return Err(e.into());
                    }
                    Err(_) => {
                        error!("Read timeout for {}", addr);
                        return Err(anyhow::anyhow!("Read timeout"));
                    }
                }
            }
            result = broadcast_rx.recv() => {
                match result {
                    Ok(message) => {
                        debug!("Sending to {}: {}", addr, message);
                        socket.write_all(message.as_bytes()).await?;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Broadcast channel closed for {}", addr);
                        return Ok(());
                    }
                    Err(e) => {
                        error!("Broadcast receive error for {}: {:?}", addr, e);
                        return Err(e.into());
                    }
                }
            }
        }
    }
}
