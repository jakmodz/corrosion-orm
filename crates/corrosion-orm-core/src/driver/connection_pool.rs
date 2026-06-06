use sqlx::FromRow;

use crate::{
    dialect::sql_dialect::SqlDialect,
    driver::{connection::Conn, executor::Executor, transaction::Transaction},
    error::CorrosionOrmError,
    query::query_type::{QueryContext, Value},
};
use std::ops::{Deref, DerefMut};

pub struct ConnectionGuard<P: ConnectionPool> {
    conn: P::Conn,
}

impl<P: ConnectionPool> ConnectionGuard<P> {
    pub(crate) fn new(conn: P::Conn) -> Self {
        Self { conn }
    }
}

impl<P: ConnectionPool> Deref for ConnectionGuard<P> {
    type Target = P::Conn;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl<P: ConnectionPool> DerefMut for ConnectionGuard<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}
impl<P: ConnectionPool> Executor for ConnectionGuard<P> {
    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64, CorrosionOrmError> {
        #[cfg(feature = "log")]
        let sql = ctx.to_debug_sql(self.conn.get_dialect());
        #[cfg(feature = "log")]
        log::info!("Executing SQL: {}", sql);
        let result = self.conn.execute_query(ctx).await;

        match &result {
            Ok(_value) => {
                #[cfg(feature = "log")]
                log::info!("Executed SQL Rows affected: {}", _value);
            }
            Err(_err) => {
                #[cfg(feature = "log")]
                log::warn!("Failed to execute SQL: {}. Error: {:?}", sql, _err);
            }
        }
        result
    }

    fn get_dialect(&self) -> &dyn SqlDialect {
        self.conn.get_dialect()
    }

    async fn fetch_one<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<E, CorrosionOrmError> {
        #[cfg(feature = "log")]
        log::info!(
            "Executing SQL: {}",
            ctx.to_debug_sql(self.conn.get_dialect())
        );
        let result = self.conn.fetch_one(ctx).await;
        #[cfg(feature = "log")]
        let sql = ctx.to_debug_sql(self.conn.get_dialect());
        match &result {
            Ok(_value) => {
                #[cfg(feature = "log")]
                log::info!("Fetched one: {}", sql);
            }
            Err(_err) => {
                #[cfg(feature = "log")]
                log::warn!("Failed to fetch one: {}. Error: {:?}", sql, _err);
            }
        }
        result
    }

    async fn fetch_all<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Vec<E>, CorrosionOrmError> {
        #[cfg(feature = "log")]
        log::info!(
            "Fetching all: {}",
            ctx.to_debug_sql(self.conn.get_dialect())
        );
        let result = self.conn.fetch_all(ctx).await;
        #[cfg(feature = "log")]
        let sql = ctx.to_debug_sql(self.conn.get_dialect());
        match &result {
            Ok(_value) => {
                #[cfg(feature = "log")]
                log::info!("Fetched all: {}", sql);
            }
            Err(_err) => {
                #[cfg(feature = "log")]
                log::warn!("Failed to fetch all: {}. Error: {:?}", sql, _err);
            }
        }
        result
    }
    async fn fetch_optional<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Option<E>, CorrosionOrmError> {
        #[cfg(feature = "log")]
        let sql = ctx.to_debug_sql(self.conn.get_dialect());
        #[cfg(feature = "log")]
        log::info!("Fetching optional: {}", sql);
        let result = self.conn.fetch_optional(ctx).await;
        match &result {
            Ok(_value) => {
                #[cfg(feature = "log")]
                log::info!("Fetched optional: {}", sql);
            }
            Err(_err) => {
                #[cfg(feature = "log")]
                log::warn!("Failed to fetch optional: {}. Error: {:?}", sql, _err);
            }
        }
        result
    }

    async fn get_last_id(&mut self) -> Result<Value, CorrosionOrmError> {
        self.conn.get_last_id().await
    }
}
/// A connection pool for managing database connections.
#[trait_variant::make(ConnPool: Send)]
pub trait ConnectionPool: Sized + Send + Sync {
    type Conn: super::connection::Conn;
    type Config: super::connection_config::ConnectionConfig;
    /// Creates a new connection pool with the given configuration.
    async fn new_pool(config: Self::Config) -> Result<Self, CorrosionOrmError>;
    /// Acquires a connection from the pool.
    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self>, CorrosionOrmError>;
    /// Returns the number of active connections in the pool.
    fn active_conn(&self) -> u32;
    /// Returns the total number of connections in the pool.
    fn total_conn(&self) -> u32;
    /// Closes the connection pool.
    async fn close(self) -> Result<(), CorrosionOrmError>;
    /// Begins a new transaction. Returns a [`Transaction`] that can be used to execute queries within the transaction.
    async fn begin_transaction(&self) -> Result<Transaction<Self>, CorrosionOrmError>;
}
