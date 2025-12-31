// src/main.rs
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    wmtp_server::server::run_server().await
}
