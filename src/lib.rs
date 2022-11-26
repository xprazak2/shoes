pub mod server;
pub mod cli;
pub mod handshake;
pub mod client;

// should we use 'anyhow' instead of boxing errors?
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
