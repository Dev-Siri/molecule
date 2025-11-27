use clap::Parser;

use crate::constants::{MOLECULE_DEFAULT_ADDR, MOLECULE_DEFAULT_PORT};

/// Majestic Rust-native SQL Database.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Address to bind to, defaults to `0.0.0.0`
    #[arg(long)]
    pub addr: Option<String>,
    /// Port to bind to, defaults to `80`
    #[arg(short, long)]
    pub port: Option<u32>,
    /// Provide a string formatted `username:password` to use in the database auth gate.
    #[arg(long)]
    pub auth: Option<String>,
    /// Run the CLI for molecule along with the database.
    #[arg(long)]
    pub cli: bool,
    #[arg(long)]
    pub enable_logging: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            addr: Some(MOLECULE_DEFAULT_ADDR.to_string()),
            port: Some(MOLECULE_DEFAULT_PORT),
            auth: None,
            cli: false,
            enable_logging: false,
        }
    }
}
