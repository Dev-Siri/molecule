use std::io::Write;

use anyhow::Result;
use colored::*;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

use crate::proto::InputType;

pub async fn run(addr: &str) -> Result<()> {
    println!("{} molecule", "[INFO]".blue());
    println!("{} Database is running on tcp://{}", "[INFO]".blue(), addr);

    let mut reader = BufReader::new(tokio::io::stdin());
    let mut input = String::new();

    loop {
        input.clear();
        print!("> ");
        std::io::stdout().flush()?;

        reader.read_line(&mut input).await?;
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
    }

    println!("{} Gracefully shutting down...", "[INFO]".blue());
    Ok(())
}
