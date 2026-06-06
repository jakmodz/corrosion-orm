mod migration;
mod migraton_registry;
use std::path::Path;

use clap::{Parser, Subcommand};
#[derive(Parser)]
struct Cli {
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
    Init { path: String },
}
fn init_migration(path: &str) -> anyhow::Result<()> {
    let path = Path::new(path);
    // init entire migration "project" thing look at seaorm
    //
    Ok(())
}
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Migration(cmd) => match cmd {
            MigrationCommand::Init { path } => init_migration(&path)?,
        },
    }
    Ok(())
}
