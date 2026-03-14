use crate::{driver::executor::Executor, error::CargoOrmError};

#[trait_variant::make(Repo: Send)]
pub trait Repository<Db: Executor>: Sized + Sync {
    type PrimaryKey;

    async fn save(&self, db: &Db) -> Result<Self, CargoOrmError>;
    async fn get_all(db: &Db) -> Result<Vec<Self>, CargoOrmError>;
    async fn get_by_id(id: Self::PrimaryKey, db: &Db) -> Result<Self, CargoOrmError>;
    async fn delete_by_id(id: Self::PrimaryKey, db: &Db) -> Result<(), CargoOrmError>;
}
