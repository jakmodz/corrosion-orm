use crate::{
    CorrosionOrmError,
    driver::{
        connection_pool::{ConnectionGuard, ConnectionPool},
        sql_driver::SqlDriver,
        sqlite_driver::sqlite_connection_pool::CorrosionSqlitePool,
    },
};

pub struct SqliteDriver {
    pool: CorrosionSqlitePool,
}

impl SqlDriver for SqliteDriver {
    type Pool = CorrosionSqlitePool;

    async fn new(
        config: <Self::Pool as crate::driver::connection_pool::ConnectionPool>::Config,
    ) -> Result<Self, CorrosionOrmError> {
        let pool = CorrosionSqlitePool::new_pool(config).await?;
        Ok(Self { pool })
    }
    fn pool(&self) -> &Self::Pool {
        &self.pool
    }
    async fn transaction(
        &self,
    ) -> Result<crate::driver::transaction::Transaction<Self::Pool>, CorrosionOrmError> {
        let tx = self.pool.begin_transaction().await?;
        Ok(tx)
    }

    async fn acquire_conn(&self) -> Result<ConnectionGuard<Self::Pool>, CorrosionOrmError> {
        self.pool.acquire_conn().await
    }

    fn active_connections(&self) -> u32 {
        self.pool.active_conn()
    }
}
