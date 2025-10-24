use crate::protocol::ChatMessage;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::info;

/// A client for connecting to and interacting with the chat server.
pub struct Client {
    stream: TcpStream,
}

impl Client {
    /// Establishes a connection to the chat server at the given address.
    ///
    /// # Arguments
    /// - `addr`: The server address (e.g., "127.0.0.1:8080").
    ///
    /// # Returns
    /// A `Result` containing the `Client` or an error if connection fails.
    ///
    /// # Examples
    /// ```rust
    /// # #[tokio::test]
    /// # async fn doc_test() {
    /// let client = Client::connect("127.0.0.1:8080").await.unwrap();
    /// # }
    /// ```
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        info!("Connected to {}", addr);
        Ok(Client { stream })
    }

    /// Sends a `ChatMessage` to the server.
    ///
    /// # Arguments
    /// - `message`: The `ChatMessage` to send.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub async fn send(&mut self, message: ChatMessage) -> Result<()> {
        let json = message.to_json()?;
        self.stream.write_all(json.as_bytes()).await?;
        self.stream.write_all(b"\n").await?; // Delimit with newline
        info!("Sent: {}", json);
        Ok(())
    }

    /// Receives a message from the server.
    ///
    /// # Returns
    /// A `Result` containing the received string or an error.
    pub async fn receive(&mut self) -> Result<String> {
        let mut buffer = [0; 1024];
        let n = self.stream.read(&mut buffer).await?;
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }
}
