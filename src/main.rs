use std::process;
use std::sync::Arc;

use anyhow::{Result, bail};
use clap::Parser;
use tokio::fs;

use crate::auth::MoleculeAuthApi;
use crate::cli::MoleculeCliApi;
use crate::constants::{
    MOLECULE_DEFAULT_ADDR, MOLECULE_DEFAULT_DATA_COLLECTION_META_PATH,
    MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH, MOLECULE_DEFAULT_DATA_PATH, MOLECULE_DEFAULT_PORT,
    MOLECULE_DOT_FILE_PATH,
};
use crate::tcp::MoleculeTcpApi;
use crate::{args::Args, molecule::Molecule};

mod args;
mod auth;
mod cli;
mod constants;
mod core;
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

    if args.enable_logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("molecule"))
            .init();
        log::info!("Logging enabled.");
    }

    if !fs::try_exists(MOLECULE_DEFAULT_DATA_PATH).await? {
        fs::create_dir_all(MOLECULE_DEFAULT_DATA_PATH).await?;
    }

    if !fs::try_exists(MOLECULE_DOT_FILE_PATH).await? {
        fs::create_dir_all(MOLECULE_DOT_FILE_PATH).await?;
    }

    if !fs::try_exists(MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH).await? {
        fs::create_dir_all(MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH).await?;
    }

    if !fs::try_exists(MOLECULE_DEFAULT_DATA_COLLECTION_META_PATH).await? {
        fs::write(MOLECULE_DEFAULT_DATA_COLLECTION_META_PATH, b"[]").await?;
    }

    let addr = args.addr.unwrap_or(MOLECULE_DEFAULT_ADDR.to_string());
    let port = args.port.unwrap_or(MOLECULE_DEFAULT_PORT);

    let molecule = Molecule::new(addr, port);
    let shared_molecule = Arc::new(molecule);

    if let Some(auth_str) = args.auth {
        let Some((username, password)) = auth_str.split_once(":") else {
            bail!("Could not parse auth string for username and password.")
        };

        shared_molecule
            .setup_user(username.to_owned(), password.to_owned())
            .await?;
    }

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
