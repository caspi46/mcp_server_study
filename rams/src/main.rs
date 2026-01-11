use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;

mod mcp_server;
use mcp_server::RepoServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddr = "127.0.0.1:8766".parse()?;

    println!("Repo Analysis MCP Server running on http://127.0.0.1:8766/mcp");

    let calculator = RepoServer::new();
    let service = StreamableHttpService::new(
        move || Ok(calculator.clone()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl-c");
        })
        .await?;

    Ok(())
}
