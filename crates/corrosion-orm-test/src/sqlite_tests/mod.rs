#[cfg(test)]
mod tests {
    use corrosion_orm_core::prelude::*;
    use corrosion_orm_core::query::query_type::QueryContext;
    use corrosion_orm_core::{CorrosionOrmError, SqliteConfig, SqliteConfigBuilder, SqliteDriver};

    fn get_conf() -> SqliteConfig {
        SqliteConfigBuilder::new()
            .url(String::from(":memory:"))
            .build()
    }

    #[tokio::test]
    async fn test_new() -> Result<(), CorrosionOrmError> {
        let config = get_conf();
        let _ = SqliteDriver::new(config).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_acquire_conn() -> Result<(), CorrosionOrmError> {
        let config = get_conf();
        let driver = SqliteDriver::new(config).await?;
        let _ = driver.acquire_conn().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_transaction() -> Result<(), CorrosionOrmError> {
        let config = get_conf();
        let driver = SqliteDriver::new(config).await?;
        let mut tx = driver.transaction().await?;
        tx.execute_query(&mut QueryContext::from("SELECT 1"))
            .await?;
        tx.commit().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ping_conn() -> Result<(), CorrosionOrmError> {
        let config = get_conf();
        let driver = SqliteDriver::new(config).await?;
        let mut conn = driver.acquire_conn().await?;
        conn.ping_conn().await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_active_conn() -> Result<(), CorrosionOrmError> {
        let config = get_conf();
        let driver = SqliteDriver::new(config).await?;
        let pool = driver.pool();
        let active = pool.active_conn();
        assert_eq!(active, 0);
        let _conn = driver.acquire_conn().await?;
        let active = pool.active_conn();
        assert_eq!(active, 1);
        Ok(())
    }
    #[tokio::test]
    async fn test_conn() -> Result<(), CorrosionOrmError> {
        let config = get_conf();
        let driver = SqliteDriver::new(config).await?;
        let mut conn = driver.acquire_conn().await?;
        conn.execute_query(&mut QueryContext::from("SELECT 1"))
            .await?;
        Ok(())
    }
}
