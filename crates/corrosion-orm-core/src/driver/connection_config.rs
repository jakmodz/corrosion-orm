use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionConfigError {
    #[error("Url to database is empty")]
    EmptyUrl,
    #[error("Env file not found")]
    EnvFileNotFound,
}

/// Configuration for a database connection.
pub trait ConnectionConfig {
    /// Returns the URL to the database.
    fn url(&self) -> &str;
    /// Returns the maximum number of connections.
    fn max_connections(&self) -> usize {
        10
    }
    /// Returns the minimum number of connections.
    fn min_connections(&self) -> usize {
        1
    }
    /// Returns the connection timeout in milliseconds.
    fn connection_timeout_ms(&self) -> u64 {
        5000
    }

    fn validate(&self) -> Result<(), ConnectionConfigError> {
        if self.url().is_empty() {
            return Err(ConnectionConfigError::EmptyUrl);
        }
        Ok(())
    }
    fn from_env() -> Result<Self, ConnectionConfigError>
    where
        Self: Sized;
}
