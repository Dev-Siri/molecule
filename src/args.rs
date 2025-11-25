use clap::Parser;

use crate::constants::{MOLECULE_DEFAULT_ADDR, MOLECULE_DEFAULT_DATA_PATH, MOLECULE_DEFAULT_PORT};

/// Majestic Rust-native SQL Database.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Path to where molecule will store it's data, defaults to `~/.molecule/data`
    #[arg(long)]
    pub data: Option<String>,
    /// Address to bind to, defaults to `0.0.0.0`
    #[arg(long)]
    pub addr: Option<String>,
    /// Port to bind to, defaults to `80`
    #[arg(short, long)]
    pub port: Option<u32>,
    /// Run the CLI for molecule along with the database.
    #[arg(long)]
    pub cli: bool,
    /// Enable logging database activity.
    #[arg(long)]
    pub enable_logging: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            data: Some(MOLECULE_DEFAULT_DATA_PATH.to_string()),
            addr: Some(MOLECULE_DEFAULT_ADDR.to_string()),
            port: Some(MOLECULE_DEFAULT_PORT),
            cli: false,
            enable_logging: false,
        }
    }
}
