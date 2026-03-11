use crate::{
    driver::{
        connection_pool::{ConnectionGuard, ConnectionPool},
        transaction::Transaction,
    },
    error::CargoOrmError,
};
/// Represents a SQL driver that provides connection pooling and transaction management.
#[trait_variant::make(SqlDriv: Send)]
pub trait SqlDriver: Sized + Sync + Send {
    type Pool: ConnectionPool;

    async fn new(config: <Self::Pool as ConnectionPool>::Config) -> Result<Self, CargoOrmError>;
    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self::Pool>, CargoOrmError>;

    fn active_connections(&self) -> u32;

    fn pool(&self) -> &Self::Pool;
    async fn transaction(&self) -> Result<Transaction<Self::Pool>, CargoOrmError>;
}
