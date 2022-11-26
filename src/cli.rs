use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long)]
    pub port: Option<u16>,
}

#[derive(Parser, Debug)]
pub struct ClientCli {
    #[clap(short, long)]
    pub port: Option<u16>,

    #[clap(short, long)]
    pub host: Option<String>,
}
