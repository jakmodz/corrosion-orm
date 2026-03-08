use crate::{
    driver::{
        connection::Connection,
        connection_pool::{ConnectionGuard, ConnectionPool},
    },
    error::CargoOrmError,
};

pub trait SqlDriver: Sized + Sync + Send {
    type Pool: ConnectionPool;

    async fn new(config: <Self::Pool as ConnectionPool>::Config) -> Result<Self, CargoOrmError>;

    async fn execute(&self, sql: &str) -> Result<u64, CargoOrmError> {
        let guard = self.acquire_conn().await?;
        guard.execute(sql).await
    }
    async fn acquire_conn(&self) -> Result<ConnectionGuard<'_, Self::Pool>, CargoOrmError> {
        self.pool().acquire_conn().await
    }
    async fn ping(&self) -> Result<(), CargoOrmError>;
    fn active_connections(&self) -> u32 {
        self.pool().active_conn()
    }
    fn pool(&self) -> &Self::Pool;
}
