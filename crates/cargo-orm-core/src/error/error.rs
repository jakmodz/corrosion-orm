use crate::schema::table::SchemaValidationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CargoOrmError {
    #[error(transparent)]
    SchemaValidationErrors(#[from] SchemaValidationError),
}
