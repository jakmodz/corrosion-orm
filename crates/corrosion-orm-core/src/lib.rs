pub mod dialect;
pub mod driver;
pub mod error;
pub mod model;
pub mod query;
pub mod schema;
pub mod types;
pub mod validation;
pub use once_cell;

pub use driver::executor::Executor;
pub use error::CorrosionOrmError;

#[cfg(feature = "sqlite")]
pub use driver::sqlite_driver::{
    sqlite_config::{SqliteConfig, SqliteConfigBuilder},
    sqlite_connection::CorrosionSqliteConnection,
    sqlite_connection_pool::CorrosionSqlitePool,
    sqlite_driver_impl::SqliteDriver,
};
#[cfg(feature = "sqlite")]
pub use sqlx;

pub use model::repository::Repository;

#[cfg(feature = "log")]
pub use log as log_crate;
#[cfg(feature = "log")]
pub use log::{debug, error, info, trace, warn};

pub mod prelude {
    pub use crate::driver::connection::Connection;
    pub use crate::driver::connection_config::ConnectionConfig;
    pub use crate::driver::connection_pool::ConnectionPool;
    pub use crate::driver::executor::Executor;
    pub use crate::driver::sql_driver::{SqlDriv, SqlDriver};
    pub use crate::driver::transaction::Transaction;
    pub use crate::error::CorrosionOrmError;
    pub use crate::model::repository::Repo;
    pub use crate::model::{CursorPaginator, Paginator};
    pub use crate::model::{Lazy, LazyCollection};
    pub use crate::query::*;
    pub use crate::query::{
        Create, Delete, Insert, Select, Update,
        query_type::{QueryContext, Value},
        to_sql::ToSql,
    };
    pub use crate::schema::table::{TableSchema, TableSchemaModel};
}
