use std::collections::HashSet;

use anyhow::{Result, anyhow};
use corrosion_orm::{Create, TableSchema};

use crate::{
    migration::{MigrationExecutor, MigratorTrait},
    migration_registry::MigrationRegistry,
};

pub struct MigrationStatus {
    pub name: String,
    pub applied: bool,
}

pub async fn ensure_registry_table(db: &mut dyn MigrationExecutor) -> Result<()> {
    let create = Create::from(MigrationRegistry::get_schema()).if_not_exists();
    db.execute_create_query(&create).await?;
    Ok(())
}

pub async fn up<M: MigratorTrait>(
    db: &mut dyn MigrationExecutor,
    steps: Option<usize>,
) -> Result<usize> {
    ensure_registry_table(db).await?;
    if steps == Some(0) {
        return Ok(0);
    }
    let applied = db.fetch_applied_migration_names().await?;
    let applied_set: HashSet<String> = applied.into_iter().collect();

    let mut applied_count = 0usize;

    for migration in M::migrations() {
        if applied_set.contains(migration.name()) {
            continue;
        }

        migration.up(db).await?;
        db.insert_applied_migration(migration.name(), chrono::Local::now().naive_local())
            .await?;

        applied_count += 1;

        if steps.is_some_and(|s| applied_count >= s) {
            break;
        }
    }

    Ok(applied_count)
}

pub async fn down<M: MigratorTrait>(
    db: &mut dyn MigrationExecutor,
    steps: Option<usize>,
) -> Result<usize> {
    ensure_registry_table(db).await?;
    if steps == Some(0) {
        return Ok(0);
    }
    let applied = db.fetch_applied_migration_names().await?;
    if applied.is_empty() {
        return Ok(0);
    }

    let mut rolled_back = 0usize;
    let mut all_migrations = M::migrations();

    for applied_name in applied.iter().rev() {
        let Some(migration) = all_migrations.iter_mut().find(|m| m.name() == applied_name) else {
            return Err(anyhow!(
                "Applied migration '{}' is missing from Migrator::migrations()",
                applied_name
            ));
        };

        migration.down(db).await?;
        db.delete_applied_migration(applied_name).await?;

        rolled_back += 1;
        if steps.is_some_and(|s| rolled_back >= s) {
            break;
        }
    }

    Ok(rolled_back)
}

pub async fn status<M: MigratorTrait>(
    db: &mut dyn MigrationExecutor,
) -> Result<Vec<MigrationStatus>> {
    ensure_registry_table(db).await?;

    let applied = db.fetch_applied_migration_names().await?;
    let applied_set: HashSet<String> = applied.into_iter().collect();

    Ok(M::migrations()
        .into_iter()
        .map(|m| MigrationStatus {
            name: m.name().to_string(),
            applied: applied_set.contains(m.name()),
        })
        .collect())
}
