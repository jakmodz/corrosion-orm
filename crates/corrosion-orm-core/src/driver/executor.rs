use sqlx::FromRow;

use crate::{
    dialect::sql_dialect::SqlDialect, error::CorrosionOrmError, query::query_type::QueryContext,
};

/// Executes SQL queries with a database entity.
#[trait_variant::make(Executor: Send)]
pub trait LocalExecutor: Sized + Send + Sync {
    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64, CorrosionOrmError>;
    async fn fetch_one<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<E, CorrosionOrmError>;
    async fn fetch_all<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Vec<E>, CorrosionOrmError>;
    async fn fetch_optional<E: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Option<E>, CorrosionOrmError>;

    fn get_dialect(&self) -> &dyn SqlDialect;
}
