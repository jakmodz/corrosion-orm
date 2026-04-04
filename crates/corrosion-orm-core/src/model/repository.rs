use crate::{driver::executor::Executor, error::CorrosionOrmError};

#[trait_variant::make(Repo: Send)]
pub trait Repository<Db: Executor>: Sized + Sync {
    /// The primary key type for this repository.
    type PrimaryKey;

    async fn save(&self, db: &mut Db) -> Result<Self, CorrosionOrmError>;
    async fn get_all(db: &mut Db) -> Result<Vec<Self>, CorrosionOrmError>;
    async fn get_by_id(
        id: Self::PrimaryKey,
        db: &mut Db,
    ) -> Result<Option<Self>, CorrosionOrmError>;
    async fn delete(self, db: &mut Db) -> Result<(), CorrosionOrmError>;
}
