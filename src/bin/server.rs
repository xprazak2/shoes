use tokio::net::TcpListener;

use shoes::server;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let port = "7474";
  let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

  server::run(listener).await;
  Ok(())
}