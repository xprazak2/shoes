use clap::Parser;
use tokio::net::TcpListener;

use shoes::{server, cli::Cli};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let default_port = 7474;
  let args = Cli::parse();
  let port = args.port.unwrap_or(default_port);

  let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

  server::run(listener).await;
  Ok(())
}