pub mod dialect;
pub mod driver;
pub mod error;
pub mod model;
pub mod schema;
pub mod types;

pub use driver::executor::Executor;
pub use error::CargoOrmError;

#[cfg(feature = "sqlite")]
pub use driver::sqlite_driver::{
    sqlite_config::{SqliteConfig, SqliteConfigBuilder},
    sqlite_connection::CargoSqliteConnection,
    sqlite_connection_pool::CargoSqlitePool,
    sqlite_driver::SqliteDriver,
};

pub mod prelude {
    pub use crate::driver::connection::Connection;
    pub use crate::driver::connection_config::ConnectionConfig;
    pub use crate::driver::connection_pool::ConnectionPool;
    pub use crate::driver::executor::Executor;
    pub use crate::driver::sql_driver::SqlDriver;
    pub use crate::driver::transaction::Transaction;
}
