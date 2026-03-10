use crate::error::CargoOrmError;

pub trait Connection: Sized + Sync + Send {
    async fn ping_conn(&mut self) -> Result<(), CargoOrmError>;
    //TODO query type
    async fn execute_query(&mut self, query: &str) -> Result<u64, CargoOrmError>;
    async fn is_valid(&mut self) -> bool;
}
