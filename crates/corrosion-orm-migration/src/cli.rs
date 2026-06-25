use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use corrosion_orm::model::snapshot::{ModelRegistry, ModelsSnapshot};

use crate::{
    MigratorTrait,
    driver_util::{connect, make_executor_from},
    migration_generator::create_migration,
    runner,
};

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub database_url: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Generate,
    Create {
        #[arg(long)]
        name: String,
    },
    Up {
        #[arg(long)]
        steps: Option<usize>,
    },
    Down {
        #[arg(long)]
        steps: Option<usize>,
    },
    Status,
}

pub async fn run_cli<M: MigratorTrait>(registry: &ModelRegistry) -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Create { name } => {
            let new_snapshot = registry.snapshot();
            let old_snapshot = if let Some(last_migration) = M::migrations().last() {
                last_snapshot(last_migration.name())?
            } else {
                None
            };
            let migration_name =
                create_migration(Some(&name), &old_snapshot, &new_snapshot).await?;
            save_snapshot(&migration_name, &new_snapshot)?;
            println!("Created migration: {migration_name}");
            Ok(())
        }
        Command::Generate => {
            let new_snapshot = registry.snapshot();
            let old_snapshot = if let Some(last_migration) = M::migrations().last() {
                last_snapshot(last_migration.name())?
            } else {
                None
            };
            let migration_name = create_migration(None, &old_snapshot, &new_snapshot).await?;
            save_snapshot(&migration_name, &new_snapshot)?;
            println!("Created migration: {migration_name}");
            Ok(())
        }
        Command::Up { steps } => {
            let db_url = cli
                .database_url
                .as_deref()
                .context("--database-url is required for migration up")?;
            let mut conn = connect(db_url).await?;
            let mut executor = make_executor_from(&mut conn);
            let applied = runner::up::<M>(&mut *executor, steps).await?;
            println!("Applied {applied} migration(s)");
            Ok(())
        }
        Command::Down { steps } => {
            let db_url = cli
                .database_url
                .as_deref()
                .context("--database-url is required for migration down")?;
            let mut conn = connect(db_url).await?;
            let mut executor = make_executor_from(&mut conn);
            let rolled_back = runner::down::<M>(&mut *executor, steps).await?;
            println!("Rolled back {rolled_back} migration(s)");
            Ok(())
        }
        Command::Status => {
            let db_url = cli
                .database_url
                .as_deref()
                .context("--database-url is required for migration status")?;
            let mut conn = connect(db_url).await?;
            let mut executor = make_executor_from(&mut conn);
            let statuses = runner::status::<M>(&mut *executor).await?;
            for status in statuses {
                let marker = if status.applied { "[x]" } else { "[ ]" };
                println!("{marker} {}", status.name);
            }
            Ok(())
        }
    }
}
fn last_snapshot(name: &str) -> Result<Option<ModelsSnapshot>> {
    let snapshot_file_path = format!(".corrosion/{name}_snapshot.json");
    let Ok(snapshot_content) = std::fs::read_to_string(&snapshot_file_path) else {
        return Ok(None);
    };
    if snapshot_content.trim().is_empty() {
        return Ok(None);
    }
    let old_snapshot: ModelsSnapshot = serde_json::from_str(&snapshot_content)?;
    Ok(Some(old_snapshot))
}
fn save_snapshot(name: &str, snapshot: &ModelsSnapshot) -> Result<()> {
    let snapshot_content = serde_json::to_string_pretty(snapshot)?;
    std::fs::create_dir_all(".corrosion")?;
    std::fs::write(format!(".corrosion/{name}_snapshot.json"), snapshot_content)?;
    Ok(())
}
