use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
// tokio_unstable
use tracing::{info, debug, error};

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
    /// let server = ChatServer::new("127.0.0.1:8080").await.unwrap();
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

            // Spawn a task for each client
            tokio::spawn(async move {
                if let Err(e) = handle_client(socket, addr, broadcast_tx, broadcast_rx).await {
                    error!("Client {} error: {:?}", addr, e);
                }
            });
        }
    }
}

/// Handles a single client connection.
///
/// Reads messages from the client, broadcasts them, and sends broadcasted messages
/// to the client using a `select!` loop for concurrent I/O.
///
/// # Arguments
/// - `socket`: The client’s TCP socket.
/// - `addr`: The client’s address.
/// - `broadcast_tx`: Sender for broadcasting messages.
/// - `broadcast_rx`: Receiver for broadcasted messages.
async fn handle_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    broadcast_tx: broadcast::Sender<String>,
    mut broadcast_rx: broadcast::Receiver<String>,
) -> Result<()> {
    info!("Handling client {}", addr);
    let mut buffer = [0; 1024];

    loop {
        tokio::select! {
            // Read from client
            result = socket.read(&mut buffer) => {
                match result {
                    Ok(0) => {
                        info!("Client {} disconnected", addr);
                        return Ok(());
                    }
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        if !message.is_empty() {
                            let formatted = format!("{}: {}", addr, message);
                            debug!("Broadcasting: {}", formatted);
                            broadcast_tx.send(formatted)?;
                        }
                    }
                    Err(e) => {
                        error!("Read error for {}: {:?}", addr, e);
                        return Err(e.into());
                    }
                }
            }
            // Write broadcasted messages to client
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