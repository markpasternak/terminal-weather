use anyhow::Result;
use clap::Parser;
use terminal_weather::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    terminal_weather::run(cli).await
}
