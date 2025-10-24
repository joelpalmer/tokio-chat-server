use console_subscriber;
use tokio_chat_server::ChatServer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();
    info!("Starting chat server on 127.0.0.1:8080");
    let server = ChatServer::new("127.0.0.1:8080").await?;
    server.run().await?;
    Ok(())
}
