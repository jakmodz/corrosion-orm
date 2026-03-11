use crate::{driver::executor::Executor, error::CargoOrmError};

#[trait_variant::make(Repo: Send)]
pub trait Repository: Sized + Sync {
    type PrimaryKey;
    type Db: Executor;

    async fn save(&self, db: &Self::Db) -> Result<Self, CargoOrmError>;
    async fn get_all(db: &Self::Db) -> Result<Vec<Self>, CargoOrmError>;
    async fn get_by_id(id: Self::PrimaryKey, db: &Self::Db) -> Result<Self, CargoOrmError>;
    async fn delete_by_id(id: Self::PrimaryKey, db: &Self::Db) -> Result<(), CargoOrmError>;
}
