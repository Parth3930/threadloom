use threadloom_mcp::McpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = McpServer::new();
    server.run_stdio().await?;
    Ok(())
}
