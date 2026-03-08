use crate::error::CargoOrmError;

pub trait Connection: Sized + Sync + Send {
    async fn ping(&self) -> Result<(), CargoOrmError>;
    async fn execute(&self, query: &str) -> Result<u64, CargoOrmError>;
    fn is_valid(&self) -> bool;
}
