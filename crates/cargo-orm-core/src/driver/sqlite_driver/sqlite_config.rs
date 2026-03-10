use crate::driver::connection_config::{ConnectionConfig, ConnectionConfigError};

pub struct SqliteConfig {
    pub url: String,
    pub max_connections: usize,
    pub min_connections: usize,
    pub connection_timeout_ms: u64,
}

impl ConnectionConfig for SqliteConfig {
    fn url(&self) -> &str {
        &self.url
    }
    fn max_connections(&self) -> usize {
        self.max_connections
    }
    fn min_connections(&self) -> usize {
        self.min_connections
    }
    fn connection_timeout_ms(&self) -> u64 {
        self.connection_timeout_ms
    }

    fn from_env() -> Result<Self, ConnectionConfigError>
    where
        Self: Sized,
    {
        let url = std::env::var("DATABASE_URL").unwrap_or(String::new());
        if url.is_empty() {
            return Err(ConnectionConfigError::EnvFileNotFound);
        }
        Ok(Self {
            url,
            max_connections: 1,
            min_connections: 0,
            connection_timeout_ms: 3000,
        })
    }
}
impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            url: String::from("sqlite::memory:"),
            max_connections: 1,
            min_connections: 0,
            connection_timeout_ms: 3000,
        }
    }
}
