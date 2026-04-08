use anyhow::Result;
use clap::Parser;

mod command;
use command::{Cli, CommandContext, execute};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let ctx = CommandContext::from_cli(
        cli.pdf_dir,
        cli.db_path,
        cli.ss_api_key,
        cli.proxy,
    ).await?;

    execute(&ctx, &cli.command).await?;

    Ok(())
}