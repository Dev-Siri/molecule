use anyhow::Result;

mod auth;
mod cli;
mod proto;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1";
    cli::run(addr).await?;
    Ok(())
}
