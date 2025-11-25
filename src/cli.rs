use std::io::Write;

use anyhow::Result;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::signal;

use crate::molecule::Molecule;
use crate::proto::InputType;

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
                    let parsed_input = match InputType::try_from(trimmed) {
                        Ok(pinput) => pinput,
                        Err(err) => {
                            println!("{}", err.to_string());
                            continue;
                        }
                    };

                    if matches!(parsed_input, InputType::Stop) {
                        break;
                    }
                },
                _ = signal::ctrl_c() => break,
            }
        }

        log::info!("Gracefully shutting down...");
        Ok(())
    }
}
