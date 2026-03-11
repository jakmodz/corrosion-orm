use crate::error::CargoOrmError;

/// Executes SQL queries with a database entity.
#[trait_variant::make(Exec: Send)]
pub trait Executor: Sized + Send + Sync {
    async fn execute_query(&mut self, sql: &str) -> Result<u64, CargoOrmError>;
}
