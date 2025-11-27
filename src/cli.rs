use std::io::Write;
use std::process;

use anyhow::Result;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::signal;

use crate::core::collection::MoleculeCoreCollectionApi;
use crate::molecule::Molecule;
use crate::proto::DatabaseInputType;
use crate::proto::InputSource;
use crate::proto::parse_str_to_db_input_type;

pub trait MoleculeCliApi {
    async fn start_cli(&self) -> Result<()>;
}

impl MoleculeCliApi for Molecule {
    async fn start_cli(&self) -> Result<()> {
        log::info!("Database is running on tcp://{}:{}", self.addr, self.port);

        let mut reader = BufReader::new(tokio::io::stdin());
        let mut input = String::new();

        loop {
            input.clear();
            print!("> ");
            std::io::stdout().flush()?;

            tokio::select! {
                Ok(_) = reader.read_line(&mut input) => {
                    let trimmed = input.trim();
                    let parsed_input = match parse_str_to_db_input_type(trimmed.to_string(), InputSource::Cli) {
                        Ok(pinput) => pinput,
                        Err(err) => {
                            println!("{}", err.to_string());
                            continue;
                        }
                    };

                    match parsed_input {
                        DatabaseInputType::Stop => break,
                        DatabaseInputType::CollectionsList => {
                            let collections = self.list_collections().await?;

                            if collections.is_empty() {
                                println!("No collections to list.");
                            }

                            for collection in collections {
                                println!("{}({})", collection.name, collection.collection_id);
                            }
                        },
                        DatabaseInputType::Collection(collection_id) => {
                            if let Some(collection) = self.get_collection_name(collection_id).await? {
                                println!("{}", collection);
                            } else {
                                println!("No collection found with that ID.");
                            }
                        },
                        DatabaseInputType::Noop => log::info!("Received empty (noop) operation."),
                    };
                },
                _ = signal::ctrl_c() => break,
            }
        }

        log::info!("Gracefully shutting down...");
        process::exit(0);
    }
}
