use crate::{
    CargoOrmError,
    driver::{
        connection_pool::{ConnectionGuard, ConnectionPool},
        sql_driver::SqlDriver,
        sqlite_driver::sqlite_connection_pool::CargoSqlitePool,
    },
};

pub struct SqliteDriver {
    pool: CargoSqlitePool,
}

impl SqlDriver for SqliteDriver {
    type Pool = CargoSqlitePool;

    async fn new(
        config: <Self::Pool as crate::driver::connection_pool::ConnectionPool>::Config,
    ) -> Result<Self, CargoOrmError> {
        let pool = CargoSqlitePool::new_pool(config).await?;
        Ok(Self { pool })
    }
    fn pool(&self) -> &Self::Pool {
        &self.pool
    }
    async fn transaction(
        &self,
    ) -> Result<crate::driver::transaction::Transaction<Self::Pool>, CargoOrmError> {
        let tx = self.pool.begin_transaction().await?;
        Ok(tx)
    }

    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self::Pool>, CargoOrmError> {
        self.pool.acquire_conn().await
    }

    fn active_connections(&self) -> u32 {
        self.pool.active_conn()
    }
}
