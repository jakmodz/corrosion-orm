use crate::types::{
    colum_type::{SqlType, ToSqlType},
    generation_strategy::GenerationType,
};

pub trait TableSchema {
    /// Method to get the table name
    fn get_table_name() -> &'static str;
    /// Method to get the table schema
    fn get_schema() -> TableSchemaModel;
}
/// Struct representing the table model
pub struct TableSchemaModel {
    /// Name of the table
    pub name: String,
    /// Fields of the table
    pub fields: Vec<ColumnSchemaModel>,
    /// Indexes of the table
    pub indexes: Vec<IndexModel>,
    /// Primary key of the table
    pub primary_key: PrimaryKeyModel,
}
/// Struct representing the column schema model
pub struct ColumnSchemaModel {
    /// Name of the column
    pub name: String,
    /// Whether the column is nullable or not
    pub is_nullable: bool,
    /// Whether the column is unique or not
    pub is_unique: bool,
    /// Type of the column
    pub sql_type: SqlType,
}
pub struct IndexModel {
    /// Name of the index
    pub name: String,
    /// Fields of the index
    pub fields: Vec<String>,
    /// Whether the index is unique or not
    pub unique: bool,
}
/// Struct representing the primary key model
pub struct PrimaryKeyModel {
    /// Name of the primary key
    pub name: String,
    /// Generation type of the primary key
    pub generation_type: Option<GenerationType>,
    /// Type of the primary key
    pub ty: SqlType,
}
impl ColumnSchemaModel {
    pub fn new<T: ToSqlType>(name: String, is_nullable: bool, is_unique: bool, sql_type: SqlType) -> Self {
        Self {
            name,
            is_nullable,
            is_unique,
            sql_type,
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
