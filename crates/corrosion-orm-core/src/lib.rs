pub mod dialect;
pub mod driver;
pub mod error;
pub mod model;
pub mod query;
pub mod schema;
pub mod types;

pub use driver::executor::Executor;
pub use error::CorrosionOrmError;

#[cfg(feature = "sqlite")]
pub use driver::sqlite_driver::{
    sqlite_config::{SqliteConfig, SqliteConfigBuilder},
    sqlite_connection::CorrosionSqliteConnection,
    sqlite_connection_pool::CorrosionSqlitePool,
    sqlite_driver_impl::SqliteDriver,
};
pub use model::repository::Repository;

pub mod prelude {
    pub use crate::driver::connection::Connection;
    pub use crate::driver::connection_config::ConnectionConfig;
    pub use crate::driver::connection_pool::ConnectionPool;
    pub use crate::driver::executor::Executor;
    pub use crate::driver::sql_driver::SqlDriver;
    pub use crate::driver::transaction::Transaction;
    pub use crate::error::CorrosionOrmError;
    pub use crate::model::repository::Repo;
    pub use crate::model::{CursorPaginator, Paginator};
    pub use crate::query::*;
    pub use crate::query::{
        Delete, Insert, Select, Update,
        query_type::{QueryContext, Value},
        to_sql::ToSql,
    };
    pub use crate::schema::table::{TableSchema, TableSchemaModel};
}
