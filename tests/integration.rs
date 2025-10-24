use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::time::{Duration, pause};
use tokio_chat_server::ChatServer;
use tokio_chat_server::client::Client;
use tokio_chat_server::protocol::ChatMessage;
use tracing::info;

#[tokio::test]
async fn test_chat_server() -> Result<()> {
    tracing_subscriber::fmt::init();
    pause();

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

    let mut client1 = Client::connect("127.0.0.1:8081").await?;
    let mut client2 = Client::connect("127.0.0.1:8081").await?;

    let message = ChatMessage {
        sender: "avery".to_string(),
        content: "Hello from client1".to_string(),
    };
    client1.send(message).await?;
    tokio::time::advance(Duration::from_millis(20)).await; // Time for send
    tokio::time::advance(Duration::from_millis(30)).await; // Time for process/broadcast

    let received = client2.receive().await?;
    assert!(
        received.starts_with("127.0.0.1:")
            && received.contains("\"sender\":\"avery\"")
            && received.contains("\"content\":\"Hello from client1\"")
    );

    Ok(())
}
