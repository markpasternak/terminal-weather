use anyhow::Result;
use atmos_tui::cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    atmos_tui::run(cli).await
}
