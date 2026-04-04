use crate::{driver::executor::Executor, error::CorrosionOrmError};

#[trait_variant::make(Repo: Send)]
pub trait Repository<Db: Executor>: Sized + Sync {
    /// The primary key type for this repository.
    type PrimaryKey;

    /// Saves the entity to the database.
    ///
    /// Returns the saved entity with any generated fields populated.
    async fn save(&self, db: &mut Db) -> Result<Self, CorrosionOrmError>;
    /// Returns all entities in the repository.
    ///
    /// Returns an empty vector if no entities are found.
    async fn get_all(db: &mut Db) -> Result<Vec<Self>, CorrosionOrmError>;
    /// Returns the entity with the given primary key, if one exists.
    ///
    /// Returns `Ok(None)` if no entity is found.
    async fn get_by_id(
        id: Self::PrimaryKey,
        db: &mut Db,
    ) -> Result<Option<Self>, CorrosionOrmError>;
    /// Deletes the entity from the database.
    ///
    /// Returns `Ok(())` if the entity was deleted, `Ok(None)` if no entity was found, or an error if one occurred.
    /// Note: this method consumes `self`, so it cannot be called on a borrowed value.
    async fn delete(self, db: &mut Db) -> Result<(), CorrosionOrmError>;
}
