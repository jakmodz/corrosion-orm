use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    schema::relation::RelationModel,
    types::{
        column_type::{SqlType, ToSqlType},
        generation_strategy::GenerationType,
    },
};

/// **Validation of primary keys like no primary in table or multiple of them is handled in parsing IR of table**
#[derive(Error, Debug)]
pub enum SchemaValidationError {
    #[error("Table {table_name} has multiple fields with the same name: {field_name:?}")]
    MultipleFieldsWithSameName {
        table_name: String,
        field_name: String,
    },
    #[error("Table '{table_name}' has a column with an empty name")]
    EmptyColumnName { table_name: String },
    #[error(
        "Table {0} has a primary key with generation strategy that is not supported for its type"
    )]
    UnsupportedGenerationStrategy(String),
    #[error("Table has an empty name")]
    EmptyTableName,
    #[error("Table '{table_name}': primary key name '{pk_name}' collides with a column name")]
    PrimaryKeyNameCollidesWithColumn { table_name: String, pk_name: String },
    #[error("Table '{0}': AutoIncrement requires an Integer primary key, but got {1:?}")]
    AutoIncrementRequiresInteger(String, SqlType),
    #[error("Table '{0}': primary key has an unsupported type {1:?}")]
    UnsupportedPrimaryKeyType(String, SqlType),
}

pub trait TableSchema {
    /// Method to get the table name
    fn get_table_name() -> &'static str;
    /// Method to get the table schema
    fn get_schema() -> TableSchemaModel;
}
/// Struct representing the table model
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TableSchemaModel {
    /// Name of the table
    pub name: String,
    /// Fields of the table
    pub fields: Vec<ColumnSchemaModel>,
    /// Indexes of the table
    pub indexes: Vec<IndexModel>,
    /// Primary key of the table
    pub primary_key: PrimaryKeyModel,
    /// Relations of the table
    pub relations: Vec<RelationModel>,
}
/// Struct representing the column schema model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSchemaModel {
    /// Name of the column
    pub name: String,
    /// Whether the column is nullable or not
    pub is_nullable: bool,
    /// Whether the column is unique or not
    pub is_unique: bool,
    /// Type of the column
    pub sql_type: SqlType,
    /// Generation type of the column
    pub generation_type: Option<GenerationType>,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IndexModel {
    /// Name of the index
    pub name: String,
    /// Fields of the index
    pub fields: Vec<String>,
    /// Whether the index is unique or not
    pub unique: bool,
}
/// Struct representing the primary key model
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PrimaryKeyModel {
    /// Name of the primary key
    pub name: String,
    /// Generation type of the primary key
    pub generation_type: Option<GenerationType>,
    /// Type of the primary key
    pub ty: SqlType,
}
impl TableSchemaModel {
    /// Creates a new table schema model with the given name and empty/default components.
    ///
    /// The resulting model has no fields, indexes, or relations. The primary key is initialized
    /// with an empty name, `generation_type` set to `None`, and type `SqlType::Integer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::schema::table::TableSchemaModel;
    /// use corrosion_orm_core::types::column_type::SqlType;
    ///
    /// let model = TableSchemaModel::new("users".to_string());
    /// assert_eq!(model.name, "users");
    /// assert!(model.fields.is_empty());
    /// assert!(model.indexes.is_empty());
    /// assert!(model.relations.is_empty());
    /// assert_eq!(model.primary_key.name, "");
    /// assert!(model.primary_key.generation_type.is_none());
    /// assert_eq!(model.primary_key.ty, SqlType::Integer);
    /// ```
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            indexes: Vec::new(),
            primary_key: PrimaryKeyModel {
                name: String::new(),
                generation_type: None,
                ty: SqlType::Integer,
            },
            relations: Vec::new(),
        }
    }

    pub fn column(&mut self, name: String) -> &mut Self {
        self.fields.push(ColumnSchemaModel {
            name,
            is_nullable: false,
            is_unique: false,
            sql_type: SqlType::Integer,
            generation_type: None,
        });
        self
    }
    /// Validates the table schema and returns an error if any schema rule is violated.
    ///
    /// This checks:
    /// - the table name is not empty,
    /// - the primary key type is not `Boolean`, `Float`, or `Double`,
    /// - `GenerationType::AutoIncrement` is only used with `SqlType::Integer`,
    /// - every column name is not empty and unique (column names are checked against the primary key name).
    ///
    /// # Returns
    ///
    /// `Ok(())` if all validations pass; an appropriate `SchemaValidationError` variant otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::schema::table::TableSchemaModel;
    ///
    /// let mut m = TableSchemaModel::new("users".into());
    /// // default primary key name is empty; ensure a valid pk name to pass validation in this example
    /// m.primary_key.name = "id".into();
    /// m.column("name".into());
    /// assert!(m.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), SchemaValidationError> {
        if self.name.is_empty() {
            return Err(SchemaValidationError::EmptyTableName);
        }

        let pk = &self.primary_key;

        match &pk.ty {
            SqlType::Boolean | SqlType::Float | SqlType::Double => {
                return Err(SchemaValidationError::UnsupportedPrimaryKeyType(
                    self.name.clone(),
                    pk.ty.clone(),
                ));
            }
            _ => {}
        }

        if let Some(GenerationType::AutoIncrement) = &pk.generation_type
            && pk.ty != SqlType::Integer
        {
            return Err(SchemaValidationError::AutoIncrementRequiresInteger(
                self.name.clone(),
                pk.ty.clone(),
            ));
        }

        let mut seen: HashSet<String> = HashSet::new();
        seen.insert(pk.name.clone());

        for col in &self.fields {
            if col.name.is_empty() {
                return Err(SchemaValidationError::EmptyColumnName {
                    table_name: self.name.clone(),
                });
            }
            if let Some(GenerationType::AutoIncrement) = &col.generation_type
                && col.sql_type != SqlType::Integer
            {
                return Err(SchemaValidationError::AutoIncrementRequiresInteger(
                    self.name.clone(),
                    col.sql_type.clone(),
                ));
            }
            if !seen.insert(col.name.clone()) {
                if col.name == pk.name {
                    return Err(SchemaValidationError::PrimaryKeyNameCollidesWithColumn {
                        table_name: self.name.clone(),
                        pk_name: pk.name.clone(),
                    });
                }
                return Err(SchemaValidationError::MultipleFieldsWithSameName {
                    table_name: self.name.clone(),
                    field_name: col.name.clone(),
                });
            }
        }
        Ok(())
    }
    /// Collects the table's column names: the primary key, all defined fields, and foreign-key columns from `BelongsTo` and `HasOne` relations.
    ///
    /// The returned vector preserves order: primary key first, then fields in insertion order, then matching relation foreign keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::schema::table::TableSchemaModel;
    ///
    /// let mut t = TableSchemaModel::new("users".into());
    /// t.column("name".into());
    /// let names = t.get_column_names();
    /// assert_eq!(names, vec!["", "name"]);
    /// ```
    pub fn get_column_names(&self) -> Vec<&str> {
        let mut names = Vec::with_capacity(1 + self.fields.len() + self.relations.len());
        names.push(self.primary_key.name.as_str());
        for field in &self.fields {
            names.push(field.name.as_str());
        }
        for relation in &self.relations {
            match relation.relation_type {
                crate::schema::relation::RelationType::BelongsTo
                | crate::schema::relation::RelationType::HasOne => {
                    names.push(relation.foreign_key.as_str());
                }
                _ => {}
            }
        }

        names
    }
    /// Number of columns in the table schema, counting the primary key plus all defined fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::schema::table::TableSchemaModel;
    ///
    /// let mut schema = TableSchemaModel::new("users".to_string());
    /// schema.column("name".to_string());
    /// schema.column("email".to_string());
    /// assert_eq!(schema.get_columns_len(), 3);
    /// ```
    ///
    /// # Returns
    ///
    /// The total count of columns (primary key + fields).
    pub fn get_columns_len(&self) -> usize {
        1 + self.fields.len()
    }
}

impl ColumnSchemaModel {
    pub fn new<T: ToSqlType>(
        name: String,
        is_nullable: bool,
        is_unique: bool,
        sql_type: SqlType,
        generation_type: Option<GenerationType>,
    ) -> Self {
        Self {
            name,
            is_nullable,
            is_unique,
            sql_type,
            generation_type,
        }
    }
}
impl PrimaryKeyModel {
    pub fn new<T: ToSqlType>(name: String, generation_type: Option<GenerationType>, ty: T) -> Self {
        Self {
            name,
            generation_type,
            ty: ty.to_sql_type(),
        }
    }
}
