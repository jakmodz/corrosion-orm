use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionConfigError {
    #[error("Url to database is empty")]
    EmptyUrl,
    #[error("Env file not found")]
    EnvFileNotFound,
}

pub trait ConnectionConfig {
    fn url(&self) -> &str;
    fn max_connections(&self) -> usize {
        10
    }
    fn min_connections(&self) -> usize {
        1
    }
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
