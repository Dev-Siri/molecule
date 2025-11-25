use std::process;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;

use crate::cli::MoleculeCliApi;
use crate::constants::{MOLECULE_DEFAULT_ADDR, MOLECULE_DEFAULT_PORT};
use crate::tcp::MoleculeTcpApi;
use crate::{args::Args, molecule::Molecule};

mod args;
mod auth;
mod cli;
mod constants;
mod molecule;
mod proto;
mod tcp;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        log::error!("{}", e.to_string());
        process::exit(1);
    }

    Ok(())
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let addr = args.addr.unwrap_or(MOLECULE_DEFAULT_ADDR.to_string());
    let port = args.port.unwrap_or(MOLECULE_DEFAULT_PORT);

    if args.enable_logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("molecule"))
            .init();
        log::info!("Logging enabled.");
    }

    let molecule = Molecule::new(addr, port);
    let shared_molecule = Arc::new(molecule);

    let server_handle = shared_molecule.clone();
    tokio::spawn(async move {
        if let Err(e) = server_handle.clone().start_tcp().await {
            log::error!("TCP server crashed: {e}");
        }
    });

    if args.cli {
        shared_molecule.start_cli().await?;
    }

    Ok(())
}
