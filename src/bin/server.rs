use clap::Parser;
use tokio::net::TcpListener;

use shoes::{cli::Cli, server};
use tracing::debug;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let default_port = 7474;
    let args = Cli::parse();
    let port = args.port.unwrap_or(default_port);
    let addr = format!("127.0.0.1:{}", port);

    debug!("Starting server on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    server::run(listener).await;
    Ok(())
}
