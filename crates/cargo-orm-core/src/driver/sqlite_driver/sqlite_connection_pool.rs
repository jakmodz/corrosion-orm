use sqlx::sqlite::SqlitePoolOptions;

use crate::{
    driver::{
        connection::Connection,
        connection_config::ConnectionConfig,
        connection_pool::{ConnectionGuard, ConnectionPool},
        error::DriverError,
        sqlite_driver::{sqlite_config::SqliteConfig, sqlite_connection::CargoSqliteConnection},
        transaction::Transaction,
    },
    error::CargoOrmError,
};
use std::time::Duration;
pub struct CargoSqlitePool {
    inner: sqlx::pool::Pool<sqlx::Sqlite>,
}

impl ConnectionPool for CargoSqlitePool {
    type Conn = CargoSqliteConnection;
    type Config = SqliteConfig;
    async fn new_pool(config: Self::Config) -> Result<Self, CargoOrmError> {
        config.validate()?;
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections() as u32)
            .min_connections(config.min_connections() as u32)
            .idle_timeout(Duration::from_millis(config.connection_timeout_ms()))
            .connect(config.url())
            .await
            .map_err(DriverError::Sqlx)?;

        Ok(CargoSqlitePool { inner: pool })
    }

    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self>, CargoOrmError> {
        let conn = self.inner.acquire().await.map_err(DriverError::Sqlx)?;
        Ok(ConnectionGuard::new(CargoSqliteConnection { inner: conn }))
    }

    fn active_conn(&self) -> u32 {
        self.inner.size() - self.inner.num_idle() as u32
    }

    fn total_conn(&self) -> u32 {
        self.inner.size()
    }

    async fn close(self) -> Result<(), CargoOrmError> {
        #![allow(clippy::disallowed_types)]
        sqlx::SqlitePool::close(&self.inner).await;
        Ok(())
    }
    async fn begin_transaction(&self) -> Result<Transaction<Self>, CargoOrmError> {
        let conn = self.inner.acquire().await.map_err(DriverError::Sqlx)?;
        let mut wrapped_conn = CargoSqliteConnection { inner: conn };
        wrapped_conn.begin_transaction().await?;
        Ok(Transaction::new(wrapped_conn))
    }
}
