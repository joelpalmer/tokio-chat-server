use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};
use tracing::{info, debug, error};
use crate::protocol::ChatMessage;

/// Chat server that handles client connections and message broadcasting.
///
/// Maintains a TCP listener for incoming connections and a broadcast channel
/// to send messages to all connected clients. Each client runs in a separate task.
pub struct ChatServer {
    listener: TcpListener,
    broadcast_tx: broadcast::Sender<String>,
}

impl ChatServer {
    /// Creates a new chat server bound to the given address.
    ///
    /// # Arguments
    /// - `addr`: The address to bind to (e.g., "127.0.0.1:8080").
    ///
    /// # Returns
    /// A `Result` containing the server or an error if binding fails.
    ///
    /// # Examples
    /// ```rust
    /// # #[tokio::test]
    /// # async fn doc_test() {
    /// let addr = "127.0.0.1:8082"; // Use a unique port to avoid conflicts
    /// let server = ChatServer::new(addr).await.unwrap();
    /// # }
    /// ```
    pub async fn new(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let (broadcast_tx, _) = broadcast::channel(100);
        info!("Chat server bound to {}", addr);
        Ok(ChatServer { listener, broadcast_tx })
    }

    /// Runs the server, accepting connections and spawning client handlers.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub async fn run(self) -> Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            let broadcast_tx = self.broadcast_tx.clone();
            let broadcast_rx = broadcast_tx.subscribe();
            info!("Accepted connection from {}", addr);

            tokio::spawn(async move {
                if let Err(e) = handle_client(socket, addr, broadcast_tx, broadcast_rx).await {
                    error!("Client {} error: {:?}", addr, e);
                }
            });
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
    let read_timeout = Duration::from_secs(30); // 30s timeout

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
                            let message = ChatMessage::from_raw(&raw)?;
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
                    Err(_) => { // Timeout
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