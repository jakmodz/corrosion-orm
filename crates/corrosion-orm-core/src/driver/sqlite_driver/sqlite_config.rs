use crate::driver::connection_config::{ConnectionConfig, ConnectionConfigError};

/// Configuration for the SQLite driver.
pub struct SqliteConfig {
    /// URL to the SQLite database.
    pub url: String,
    /// Maximum number of connections in the pool.
    pub max_connections: usize,
    /// Minimum number of connections in the pool.
    pub min_connections: usize,
    /// Connection timeout in milliseconds.
    pub connection_timeout_ms: u64,
}

pub struct SqliteConfigBuilder {
    url: String,
    max_connections: usize,
    min_connections: usize,
    connection_timeout_ms: u64,
}
impl Default for SqliteConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl SqliteConfigBuilder {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            max_connections: 1,
            min_connections: 0,
            connection_timeout_ms: 30000,
        }
    }
    pub fn url(self, url: String) -> Self {
        Self { url, ..self }
    }
    pub fn max_connections(self, max_connections: usize) -> Self {
        Self {
            max_connections,
            ..self
        }
    }
    pub fn min_connections(self, min_connections: usize) -> Self {
        Self {
            min_connections,
            ..self
        }
    }
    pub fn connection_timeout_ms(self, connection_timeout_ms: u64) -> Self {
        Self {
            connection_timeout_ms,
            ..self
        }
    }
    pub fn build(self) -> SqliteConfig {
        SqliteConfig {
            url: self.url,
            max_connections: self.max_connections,
            min_connections: self.min_connections,
            connection_timeout_ms: self.connection_timeout_ms,
        }
    }
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
        let url = std::env::var("DATABASE_URL").unwrap_or_default();
        if url.is_empty() {
            return Err(ConnectionConfigError::EnvFileNotFound);
        }
        let max_connections = std::env::var("MAX_CONNECTIONS")
            .unwrap_or("1".to_string())
            .parse()
            .unwrap_or(1);
        let min_connections = std::env::var("MIN_CONNECTIONS")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0);
        let connection_timeout_ms = std::env::var("CONNECTION_TIMEOUT_MS")
            .unwrap_or("3000".to_string())
            .parse()
            .unwrap_or(3000);
        Ok(Self {
            url,
            max_connections,
            min_connections,
            connection_timeout_ms,
        })
    }
}
impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            url: String::from("sqlite::memory:"),
            max_connections: 1,
            min_connections: 0,
            connection_timeout_ms: 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        let config = SqliteConfig::default();
        assert_eq!(config.url, "sqlite::memory:");
        assert_eq!(config.max_connections, 1);
        assert_eq!(config.min_connections, 0);
        assert_eq!(config.connection_timeout_ms, 5000);
    }
    #[test]
    fn test_builder() {
        let config = SqliteConfigBuilder::new()
            .url("sqlite::memory:".to_string())
            .max_connections(1)
            .min_connections(0)
            .connection_timeout_ms(5000)
            .build();
        assert_eq!(config.url, "sqlite::memory:");
        assert_eq!(config.max_connections, 1);
        assert_eq!(config.min_connections, 0);
        assert_eq!(config.connection_timeout_ms, 5000);
    }
}
