use crate::{
    driver::{executor::Executor, sql_driver::SqlDriver},
    error::CargoOrmError,
};

pub trait Repository: Sized + Sync {
    type PrimaryKey;
    type Db: Executor;

    async fn save<'db>(&self, db: &'db Self::Db) -> Result<Self, CargoOrmError>;
    async fn get_all<'db>(db: &'db Self::Db) -> Result<Vec<Self>, CargoOrmError>;
    async fn get_by_id<'db>(id: Self::PrimaryKey, db: &'db Self::Db)
    -> Result<Self, CargoOrmError>;
    async fn delete_by_id<'db>(
        id: Self::PrimaryKey,
        db: &'db Self::Db,
    ) -> Result<(), CargoOrmError>;
}
