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
        let mut guard = self.acquire_conn().await?;
        guard.execute_query(sql).await
    }

    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self::Pool>, CargoOrmError> {
        self.pool().acquire_conn().await
    }

    fn active_connections(&self) -> u32 {
        self.pool().active_conn()
    }

    fn pool(&self) -> &Self::Pool;
}
