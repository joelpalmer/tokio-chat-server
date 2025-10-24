use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Barrier;
use tokio_chat_server::ChatServer;
use tracing::info;

#[tokio::test]
async fn test_chat_server() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Create a barrier for 2 parties (server and test tasks) to sync
    let barrier = Arc::new(Barrier::new(2));
    let server_barrier = barrier.clone();

    let server = ChatServer::new("127.0.0.1:8081").await?;
    tokio::spawn(async move {
        info!("Server waiting at barrier");
        server_barrier.wait().await;
        info!("Server proceeding after barrier");
        server.run().await.unwrap();
    });

    info!("Test waiting at barrier");
    barrier.wait().await;
    info!("Test proceeding after barrier");

    let mut client1 = TcpStream::connect("127.0.0.1:8081").await?;
    let mut client2 = TcpStream::connect("127.0.0.1:8081").await?;

    let message = "Hello from client1\n";
    client1.write_all(message.as_bytes()).await?;

    let mut buffer = [0; 1024];
    let n = client2.read(&mut buffer).await?;
    let received = String::from_utf8_lossy(&buffer[..n]);
    assert!(received.contains("Hello from client1"));

    Ok(())
}
