use anyhow::Result;
use argusflow::command::{Cli, CommandContext, execute};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
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