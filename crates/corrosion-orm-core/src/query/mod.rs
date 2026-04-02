//! SQL Query builders for constructing type-safe queries.
//!
//! This module provides a fluent builder API for constructing SQL queries:
//! - [`Select`] - SELECT queries with WHERE and LIMIT support
//! - [`Insert`] - INSERT queries with column/value binding
//! - [`Update`] - UPDATE queries with WHERE conditions
//! - [`Delete`] - DELETE queries with WHERE conditions
//!
//! # Builder Pattern
//!
//! All query builders use the fluent builder pattern for chainable construction:
//!
//! ```
//! use corrosion_orm_core::query::select::Select;
//! let query = Select::new("users")
//!     .add_column("id")
//!     .add_column("name")
//!     .limit(10);
//! ```
//!
//! # TableSchemaModel Conversion
//!
//! All query builders support implicit conversion from `&TableSchemaModel`,
//! automatically extracting table name and columns:
//!
//! ```
//! use corrosion_orm_core::schema::table::TableSchemaModel;
//! use corrosion_orm_core::query::select::Select;
//! let mut schema = TableSchemaModel::new("users".to_string());
//! schema.column("id".to_string()).column("name".to_string());
//! let query = Select::from(&schema);  // Extracts table name and columns
//! ```
//! use corrosion_orm_core::schema::table::TableSchemaModel;
//! use corrosion_orm_core::query::select::Select;
//! let mut schema = TableSchemaModel::new("users".to_string());
//! schema.column("id".to_string()).column("name".to_string());
//! let query = Select::from(&schema);  // Extracts table name and columns
//! # Converting to SQL
//!
//! Use the [`ToSql`] trait to convert builders to SQL strings:
//!
//! ```
//! use corrosion_orm_core::query::to_sql::ToSql;
//! use corrosion_orm_core::query::query_type::QueryContext;
//! use corrosion_orm_core::dialect::sql_dialect::SqlDialect;
//! // let dialect = ...;
//! // let mut ctx = QueryContext::new();
//! // query.to_sql(&mut ctx, &dialect);
//! ```

pub mod delete;
pub mod insert;
pub mod query_type;
pub mod select;
pub mod to_sql;
pub mod update;
pub mod where_clause;

pub use delete::Delete;
pub use insert::Insert;
pub use select::Select;
pub use to_sql::ToSql;
pub use update::Update;
pub use where_clause::WhereClause;
