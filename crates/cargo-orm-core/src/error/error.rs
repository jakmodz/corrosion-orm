use crate::{
    driver::{connection_config::ConnectionConfigError, error::DriverError},
    schema::table::SchemaValidationError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CargoOrmError {
    #[error(transparent)]
    SchemaValidationErrors(#[from] SchemaValidationError),
    #[error(transparent)]
    DriverError(#[from] DriverError),
}
impl From<ConnectionConfigError> for CargoOrmError {
    fn from(e: ConnectionConfigError) -> Self {
        CargoOrmError::DriverError(DriverError::InvalidConfiguration(e))
    }
}
