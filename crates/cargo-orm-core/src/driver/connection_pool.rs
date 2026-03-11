use crate::{
    driver::{connection::Connection, executor::Executor, transaction::Transaction},
    error::CargoOrmError,
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
    async fn execute_query(&mut self, sql: &str) -> Result<u64, CargoOrmError> {
        self.conn.execute_query(sql).await
    }
}
/// A connection pool for managing database connections.
#[trait_variant::make(ConnPool: Send)]
pub trait ConnectionPool: Sized + Send + Sync {
    type Conn: super::connection::Connection;
    type Config: super::connection_config::ConnectionConfig;
    /// Creates a new connection pool with the given configuration.
    async fn new_pool(config: Self::Config) -> Result<Self, CargoOrmError>;
    /// Acquires a connection from the pool.
    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self>, CargoOrmError>;
    /// Returns the number of active connections in the pool.
    fn active_conn(&self) -> u32;
    /// Returns the total number of connections in the pool.
    fn total_conn(&self) -> u32;
    /// Closes the connection pool.
    async fn close(self) -> Result<(), CargoOrmError>;
    /// Begins a new transaction. Returns a [`Transaction`] that can be used to execute queries within the transaction.
    async fn begin_transaction(&self) -> Result<Transaction<Self>, CargoOrmError>;
}
