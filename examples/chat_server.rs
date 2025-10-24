use tokio_chat_server::ChatServer;
use tokio_chat_server::run_server;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with a custom format
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .init();
    info!("Starting chat server on 127.0.0.1:8080");
    let server = ChatServer::new("127.0.0.1:8080").await?;
    run_server(server.run()).await;
    Ok(())
}
