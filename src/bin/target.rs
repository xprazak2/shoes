use clap::Parser;
use tokio::net::TcpListener;
use tokio::io::{BufReader, AsyncBufReadExt};

use shoes::cli::Cli;
use tracing::{info, error};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
  tracing_subscriber::fmt::init();

  let default_port = 6666;
  let args = Cli::parse();
  let port = args.port.unwrap_or(default_port);

  let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
  info!("Running on port {}", port);

  loop {
    let (mut socket, _addr) = listener.accept().await?;

    tokio::spawn(async move {
      let (reader, _writer) = socket.split();

      let mut reader = BufReader::new(reader);
      let mut line = String::new();

      loop {
        tokio::select! {
          incoming = reader.read_line(&mut line) => {
            info!("Receiving: {:?}, incoming: {:?}", line, incoming);
            let n_read = match incoming {
              Ok(x) => x,
              Err(msg) => {
                error!("Error reading incoming stream: {}", msg);
                0
              },
            };

            if n_read == 0 {
              info!("0B read, closing connection");
              break;
            }
            info!("Received: {}", line);
            line.clear();
          }
        }
      }
    });
  }
}
