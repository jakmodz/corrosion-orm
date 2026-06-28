//! Driver module for database connections and operations.
//! This module provides the core driver implementation for database connections, including connection pooling, transactions, and SQL execution.
pub mod connection;
pub mod connection_config;
pub mod connection_pool;
pub mod db_row;
pub mod error;
pub mod executor;
pub mod from_row_db;
pub mod sql_driver;
#[cfg(feature = "sqlite")]
pub mod sqlite_driver;
pub mod transaction;
