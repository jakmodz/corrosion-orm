use crate::error::CargoOrmError;

/// Trait for single database connection.
#[trait_variant::make(Conn: Send)]
pub trait Connection: Sized + Sync + Send {
    /// Pings the connection to check if it is still valid.
    async fn ping_conn(&mut self) -> Result<(), CargoOrmError>;
    /// Executes a query and returns the number of affected rows.
    async fn execute_query(&mut self, sql: &str) -> Result<u64, CargoOrmError>;
    //TODO query type

    /// Begins a new transaction.
    async fn begin_transaction(&mut self) -> Result<(), CargoOrmError>;
    /// Commits the current transaction.
    async fn commit_transaction(&mut self) -> Result<(), CargoOrmError>;
    /// Rolls back the current transaction.
    async fn rollback_transaction(&mut self) -> Result<(), CargoOrmError>;
    /// Checks if the connection is still valid.
    async fn is_valid(&mut self) -> bool;

    fn rollback_blocking(&mut self) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = self.rollback_transaction().await;
            });
        });
    }
}
