#[allow(clippy::disallowed_types)]
use sqlx::Connection;
use sqlx::FromRow;

use crate::{
    dialect::{sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect},
    driver::error::DriverError,
    error::CorrosionOrmError,
    query::query_type::{QueryContext, Value},
};
pub struct CorrosionSqliteConnection {
    pub(crate) inner: sqlx::pool::PoolConnection<sqlx::Sqlite>,
}

impl crate::driver::connection::Conn for CorrosionSqliteConnection {
    async fn ping_conn(&mut self) -> Result<(), CorrosionOrmError> {
        self.inner.ping().await.map_err(DriverError::Sqlx)?;
        Ok(())
    }

    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64, CorrosionOrmError> {
        let mut query = sqlx::query(&ctx.sql);

        for value in ctx.values.iter() {
            query = match value {
                Value::String(s) => query.bind(s),
                Value::Int(i) => query.bind(i),
                Value::Int64(i) => query.bind(i),
                Value::Bool(b) => query.bind(b),
                Value::Date(d) => query.bind(d.to_string()),
                Value::DateTime(d) => query.bind(d.to_string()),
            }
        }

        let result = query
            .execute(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?
            .rows_affected();
        Ok(result)
    }

    async fn begin_transaction(&mut self) -> Result<(), CorrosionOrmError> {
        sqlx::query("BEGIN IMMEDIATE")
            .execute(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        Ok(())
    }

    async fn commit_transaction(&mut self) -> Result<(), CorrosionOrmError> {
        sqlx::query("COMMIT")
            .execute(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        Ok(())
    }

    async fn rollback_transaction(&mut self) -> Result<(), CorrosionOrmError> {
        sqlx::query("ROLLBACK")
            .execute(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        Ok(())
    }

    async fn is_valid(&mut self) -> bool {
        self.inner.ping().await.is_ok()
    }

    fn get_dialect(&self) -> &dyn SqlDialect {
        &SqliteDialect
    }

    async fn fetch_one<T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<T, CorrosionOrmError> {
        let mut query = sqlx::query_as::<_, T>(&ctx.sql);

        for value in ctx.values.iter() {
            query = match value {
                Value::String(s) => query.bind(s),
                Value::Int(i) => query.bind(i),
                Value::Int64(i) => query.bind(i),
                Value::Bool(b) => query.bind(b),
                Value::Date(d) => query.bind(d.to_string()),
                Value::DateTime(d) => query.bind(d.to_string()),
            }
        }

        let result = query
            .fetch_one(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        Ok(result)
    }

    async fn fetch_all<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Vec<E>, CorrosionOrmError> {
        let mut query = sqlx::query_as::<_, E>(&ctx.sql);

        for value in ctx.values.iter() {
            query = match value {
                Value::String(s) => query.bind(s),
                Value::Int(i) => query.bind(i),
                Value::Int64(i) => query.bind(i),
                Value::Bool(b) => query.bind(b),
                Value::Date(d) => query.bind(d.to_string()),
                Value::DateTime(d) => query.bind(d.to_string()),
            }
        }

        let result = query
            .fetch_all(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        Ok(result)
    }

    async fn fetch_optional<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Option<E>, CorrosionOrmError> {
        let mut query = sqlx::query_as::<_, E>(&ctx.sql);

        for value in ctx.values.iter() {
            query = match value {
                Value::String(s) => query.bind(s),
                Value::Int(i) => query.bind(i),
                Value::Int64(i) => query.bind(i),
                Value::Bool(b) => query.bind(b),
                Value::Date(d) => query.bind(d.to_string()),
                Value::DateTime(d) => query.bind(d.to_string()),
            }
        }

        let result = query
            .fetch_optional(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        Ok(result)
    }
}
