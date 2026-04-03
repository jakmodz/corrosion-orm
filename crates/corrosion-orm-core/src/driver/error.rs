use thiserror::Error;

use crate::driver::connection_config::ConnectionConfigError;

#[derive(Error, Debug)]
pub enum DriverError {
    #[error("database driver not supported")]
    UnsupportedDriver,
    #[error(transparent)]
    InvalidConfiguration(#[from] ConnectionConfigError),
    #[cfg(feature = "sqlite")]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("connection closed")]
    ConnectionClosed,
    #[error("resource not found")]
    NotFound,
}
