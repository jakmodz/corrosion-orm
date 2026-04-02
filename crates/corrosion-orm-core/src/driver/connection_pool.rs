use crate::{
    dialect::sql_dialect::SqlDialect,
    driver::{connection::Connection, executor::Executor, transaction::Transaction},
    error::CorrosionOrmError,
    query::query_type::QueryContext,
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
        self.conn.execute_query(ctx).await
    }

    fn get_dialect(&self) -> &dyn SqlDialect {
        self.conn.get_dialect()
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
