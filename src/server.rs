use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};
use tracing::debug;

use std::str;
use std::io::Cursor;

use crate::handshake::{ConnStateBuilder};


#[derive(Debug)]
struct Server {
  listener: TcpListener
}

impl Server {
  async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    // info!("accepting inbound connections");

    loop {
      let socket = self.accept().await?;

      let mut handler = ConnHandler {
        socket
      };

      tokio::spawn(async move {
        if let Err(err) = handler.run().await {
          println!("conn errror");
        }
      });
    }
  }

  async fn accept(&mut self) -> Result<TcpStream, Box<dyn std::error::Error>> {
    match self.listener.accept().await {
      Ok((socket, _)) => Ok(socket),
      Err(err) => Err(err.into()),
    }
  }
}


#[derive(Debug)]
struct ConnHandler {
  socket: TcpStream
}

impl ConnHandler {

  #[tracing::instrument(skip(self))]
  async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    println!("processing connection...");
    let res = self.read_handshake().await?;
    

    Ok(())
  }

  async fn read_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0_u8; 1024];
    let mut n_read = 0;
    let mut conn_builder = ConnStateBuilder::new();

    let req = loop {
      n_read += self.socket.read(&mut buf[n_read..]).await?;
      let msg = format!("Accepted: {:?}", String::from_utf8_lossy(&buf[0..n_read]));
      debug!(msg);
      // return reply instead of keeping it in state?
      let res = conn_builder.advance(&buf);

      match res {
        Ok(_) => self.reply(conn_builder.reply()).await?,
        Err(err) => return Err(Box::new(err)),
      }

    };
  }

  async fn reply(&mut self, reply: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if !reply.is_empty() {
      self.socket.write(&reply).await?;
    }
    Ok(())
  }

}

pub async fn run(listener: TcpListener) {
  let mut server = Server { listener };

  tokio::select! {
    res = server.run() => {
      if let Err(err) = res {
          // error!(cause = %err, "running server failed");
          println!("running server failed");
      }
    }
  }
}