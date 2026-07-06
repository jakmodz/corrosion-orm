use crate::{
    dialect::sql_dialect::SqlDialect,
    driver::from_row_db::FromRowDb,
    error::CorrosionOrmError,
    query::query_type::{QueryContext, Value},
};

/// Executes SQL queries with a database entity.
#[trait_variant::make(Executor: Send)]
pub trait LocalExecutor: Sized + Send + Sync {
    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64, CorrosionOrmError>;
    async fn fetch_one<E: FromRowDb + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<E, CorrosionOrmError>;
    async fn fetch_all<E: FromRowDb + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Vec<E>, CorrosionOrmError>;
    async fn fetch_optional<E: FromRowDb + Send + Unpin>(
        &mut self,
        ctx: &mut QueryContext,
    ) -> Result<Option<E>, CorrosionOrmError>;
    async fn get_last_id(&mut self) -> Result<Value, CorrosionOrmError>;
    fn get_dialect(&self) -> &dyn SqlDialect;
    fn cache_scope(&self) -> usize {
        self as *const Self as usize
    }
}
