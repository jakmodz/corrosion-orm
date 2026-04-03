use crate::{
    dialect::sql_dialect::SqlDialect, error::CorrosionOrmError, query::query_type::QueryContext,
};
use sqlx::FromRow;

/// Trait for single database connection.
#[trait_variant::make(Conn: Send)]
pub trait Connection: Sized + Sync + Send {
    /// Pings the connection to check if it is still valid.
    async fn ping_conn(&mut self) -> Result<(), CorrosionOrmError>;
    /// Executes a query and returns the number of affected rows.
    ///
    /// Returns the number of rows affected by the query.
    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64, CorrosionOrmError>;
    /// Fetches a single row from the result set.
    ///
    /// Returns a single row that implements the `FromRow` trait for the given row type.
    async fn fetch_one<T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<T, CorrosionOrmError>;
    /// Fetches all rows from the result set.
    ///
    /// Returns a `Vec` of rows that implement the `FromRow` trait for the given row type.
    async fn fetch_all<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Vec<E>, CorrosionOrmError>;
    /// Fetches a single row from the result set, if one exists.
    ///
    /// Returns `Ok(Some(row))` if a row is found, `Ok(None)` if no rows are found, or an error if one occurs.
    async fn fetch_optional<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Option<E>, CorrosionOrmError>;
    /// Begins a new transaction.
    async fn begin_transaction(&mut self) -> Result<(), CorrosionOrmError>;
    /// Commits the current transaction.
    async fn commit_transaction(&mut self) -> Result<(), CorrosionOrmError>;
    /// Rolls back the current transaction.
    async fn rollback_transaction(&mut self) -> Result<(), CorrosionOrmError>;
    /// Checks if the connection is still valid.
    async fn is_valid(&mut self) -> bool;
    /// Returns the SQL dialect for this connection.
    fn get_dialect(&self) -> &dyn SqlDialect;
    fn rollback_blocking(&mut self) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = self.rollback_transaction().await;
            });
        });
    }
}
