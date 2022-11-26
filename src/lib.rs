pub mod cli;
pub mod client;
pub mod handshake;
pub mod server;

// should we use 'anyhow' instead of boxing errors?
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
