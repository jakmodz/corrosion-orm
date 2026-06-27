#[allow(clippy::disallowed_types)]
use sqlx::Connection;
use sqlx::{Column, Row, sqlite::SqliteRow};

use crate::{
    dialect::{sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect},
    driver::{db_row::DbRow, error::DriverError, from_row_db::FromRowDb},
    error::CorrosionOrmError,
    query::query_type::{QueryContext, Value},
};
macro_rules! bind_to_query {
    ($query: expr, $value: expr) => {
        match $value {
            Value::String(s) => $query.bind(s),
            Value::Int(i) => $query.bind(i),
            Value::Int64(i) => $query.bind(i),
            Value::Float(f) => $query.bind(f),
            Value::Bool(b) => $query.bind(b),
            Value::Date(d) => $query.bind(d),
            Value::DateTime(d) => $query.bind(d),
            Value::Null => $query.bind(Option::<String>::None),
        }
    };
}
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
            query = bind_to_query!(query, value)
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

    async fn fetch_one<E: FromRowDb + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<E, CorrosionOrmError> {
        let mut query = sqlx::query(&ctx.sql);

        for value in ctx.values.iter() {
            query = bind_to_query!(query, value)
        }

        let row: SqliteRow = query
            .fetch_one(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        let db_row = row_to_db_row(&row)?;
        let result = E::from_row(&db_row)?;
        Ok(result)
    }

    async fn fetch_all<E: FromRowDb + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Vec<E>, CorrosionOrmError> {
        let mut query = sqlx::query(&ctx.sql);

        for value in ctx.values.iter() {
            query = bind_to_query!(query, value)
        }
        let rows: Vec<SqliteRow> = query
            .fetch_all(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        let mut results: Vec<E> = Vec::new();
        for row in rows.iter() {
            let row_db = row_to_db_row(row)?;
            let result = E::from_row(&row_db)?;
            results.push(result);
        }
        Ok(results)
    }

    async fn fetch_optional<E: FromRowDb + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Option<E>, CorrosionOrmError> {
        let mut query = sqlx::query(&ctx.sql);

        for value in ctx.values.iter() {
            query = bind_to_query!(query, value)
        }

        let row: Option<SqliteRow> = query
            .fetch_optional(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        if let Some(row) = row {
            let db_row = row_to_db_row(&row)?;
            let result = E::from_row(&db_row)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn get_last_id(&mut self) -> Result<Value, CorrosionOrmError> {
        let query = sqlx::query("SELECT last_insert_rowid()");
        let result = query
            .fetch_one(self.inner.as_mut())
            .await
            .map_err(DriverError::Sqlx)?;
        let last_id = result.try_get(0).map_err(DriverError::Sqlx)?;
        Ok(last_id)
    }
}
fn row_to_db_row(row: &sqlx::sqlite::SqliteRow) -> Result<DbRow, CorrosionOrmError> {
    let mut columns = Vec::new();
    for col in row.columns().iter() {
        let v: Value = row
            .try_get::<Value, &str>(col.name())
            .map_err(|e| CorrosionOrmError::DriverError(DriverError::Sqlx(e)))?;
        columns.push((col.name().to_string(), v));
    }
    Ok(DbRow::new(columns))
}
