#![allow(dead_code)]
use anyhow::Result;
use corrosion_orm::dialect::sql_dialect::SqlDialect;

/// All the machinery needed to run a database command.
pub struct DbResources<'a> {
    pub executor: Box<dyn crate::migration::MigrationExecutor + 'a>,
    pub dialect: &'a dyn SqlDialect,
}

/// Snapshot of DB state before running a migration – the driver and its connection.
pub(crate) struct DbHandle {
    #[cfg(feature = "sqlite")]
    pub(crate) conn: Box<
        corrosion_orm::driver::connection_pool::ConnectionGuard<corrosion_orm::CorrosionSqlitePool>,
    >,
}

#[cfg(feature = "sqlite")]
pub(crate) async fn connect_sqlite(url: &str) -> Result<DbHandle> {
    use corrosion_orm::{SqlDriver, SqliteConfigBuilder, SqliteDriver};
    let config = SqliteConfigBuilder::new().url(url.to_string()).build();
    let driver = SqliteDriver::new(config).await?;
    let conn = driver.acquire_conn().await?;
    Ok(DbHandle {
        conn: Box::new(conn),
    })
}

#[cfg(feature = "sqlite")]
pub(crate) fn make_executor<'a>(
    handle: &'a mut DbHandle,
) -> Box<dyn crate::migration::MigrationExecutor + 'a> {
    Box::new(crate::sqlite_executor::SqliteMigrationExecutor::new(
        &mut handle.conn,
    ))
}

#[cfg(feature = "sqlite")]
pub(crate) fn get_dialect() -> &'static dyn SqlDialect {
    &corrosion_orm::dialect::sqlite_dialect::sqlite::SqliteDialect
}

/// Open a database connection based on the currently compiled feature.
pub(crate) async fn connect(url: &str) -> Result<DbHandle> {
    #[cfg(feature = "sqlite")]
    return connect_sqlite(url).await;

    #[cfg(not(any(feature = "sqlite")))]
    panic!(
        "No database feature enabled. Add `sqlite` (or another driver) to `corrosion-orm-migration` features."
    );
}

/// Build a MigrationExecutor from an active DbHandle.
pub(crate) fn make_executor_from<'a>(
    handle: &'a mut DbHandle,
) -> Box<dyn crate::migration::MigrationExecutor + 'a> {
    #[cfg(feature = "sqlite")]
    return make_executor(handle);

    #[cfg(not(any(feature = "sqlite")))]
    panic!("No database feature enabled");
}

/// Return the dialect singleton for the compiled feature.
pub(crate) fn dialect() -> &'static dyn SqlDialect {
    #[cfg(feature = "sqlite")]
    return get_dialect();

    #[cfg(not(any(feature = "sqlite")))]
    panic!("No database feature enabled");
}
