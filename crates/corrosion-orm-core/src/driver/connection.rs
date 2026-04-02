use crate::{
    dialect::sql_dialect::SqlDialect, error::CorrosionOrmError, query::query_type::QueryContext,
};

/// Trait for single database connection.
#[trait_variant::make(Conn: Send)]
pub trait Connection: Sized + Sync + Send {
    /// Pings the connection to check if it is still valid.
    async fn ping_conn(&mut self) -> Result<(), CorrosionOrmError>;
    /// Executes a query and returns the number of affected rows.
    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64, CorrosionOrmError>;

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
