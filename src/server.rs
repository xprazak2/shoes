use tokio::net::{TcpListener, TcpStream};


#[derive(Debug)]
struct Server {
  listener: TcpListener
}

impl Server {
  async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    // info!("accepting inbound connections");

    loop {
      let socket = self.accept().await?;

      let mut handler = ConnHandler {};

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
struct ConnHandler {}

impl ConnHandler {
  async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    println!("processing connection...");
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