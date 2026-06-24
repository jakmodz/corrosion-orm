pub mod migration;
use std::path::PathBuf;

use crate::migration::init_migrations;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[arg(global = true, long)]
    database_url: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Migration(MigrationCommand),
}

#[derive(Subcommand)]
enum MigrationCommand {
    /// Initialize rust migration module directory
    Init {
        #[arg(default_value = "migrations")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Migration(cmd) => match cmd {
            MigrationCommand::Init { path } => {
                init_migrations(&path).await?;
                println!("Initialized rust migrations at '{}'", path.display());
            }
        },
    }

    Ok(())
}
