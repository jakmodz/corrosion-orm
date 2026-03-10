use std::ops::{Deref, DerefMut};

use crate::error::CargoOrmError;

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

pub trait ConnectionPool: Sized + Send + Sync {
    type Conn: super::connection::Connection;
    type Config: super::connection_config::ConnectionConfig;

    async fn new_pool(config: Self::Config) -> Result<Self, CargoOrmError>;
    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self>, CargoOrmError>;

    fn active_conn(&self) -> u32;
    fn total_conn(&self) -> u32;

    async fn close(self) -> Result<(), CargoOrmError>;
}
