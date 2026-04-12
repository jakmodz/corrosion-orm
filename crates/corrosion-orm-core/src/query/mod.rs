//! SQL Query builders for constructing type-safe queries.
//!
//! This module provides a fluent builder API for constructing SQL queries:
//! - [`Select`] - SELECT queries with WHERE and LIMIT support
//! - [`Insert`] - INSERT queries with column/value binding
//! - [`Update`] - UPDATE queries with WHERE conditions
//! - [`Delete`] - DELETE queries with WHERE conditions
//!
//! # Type-Safe Column References
//!
//! To ensure that column references are valid at compile time, query builders like
//! [`Select`], [`Update`], [`Delete`], and [`WhereClause`] require a generic type `C`
//! that implements  [`ColumnTrait`](crate::types::ColumnTrait). This prevents runtime errors caused by typos
//! in raw string queries and relies on generated schema column enums instead.
//!
//! # Builder Pattern
//! All query builders support implicit conversion from `&TableSchemaModel`,
//! automatically extracting table name and columns:
//!
//! ```
//! use corrosion_orm_core::schema::table::TableSchemaModel;
//! use corrosion_orm_core::query::select::Select;
//! ```
//! use corrosion_orm_core::schema::table::TableSchemaModel;
//! use corrosion_orm_core::query::select::Select;
//! # Converting to SQL
//! # TableSchemaModel Conversion
//!
//! All query builders support implicit conversion from `&TableSchemaModel`,
//! automatically extracting table name and columns:
//!
//! ```
//! use corrosion_orm_core::schema::table::TableSchemaModel;
//! use corrosion_orm_core::query::select::Select;
//! use corrosion_orm_core::types::ColumnTrait;
//!
//! #[derive(Clone, Copy)]
//! pub enum UserColumn {
//!     Id,
//! }
//!
//! impl ColumnTrait for UserColumn {
//!     fn as_str(&self) -> &'static str {
//!         "id"
//!     }
//! }
//!
//! // let schema = TableSchemaModel { ... };
//! // let select = Select::<UserColumn>::from(&schema);
//! ```
//!
//! # Converting to SQL
//!
//! Use the [`ToSql`] trait to convert builders to SQL strings:

pub mod delete;
pub mod insert;
pub mod order_by;
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
