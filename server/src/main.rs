// src/main.rs
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("wmtp_server=info,wtransport=trace")
        .init();
    wmtp_server::server::run_server().await
}
