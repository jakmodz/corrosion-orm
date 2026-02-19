use crate::types::colum_type::{SqlType,ToSqlType};
pub trait TableSchema{
    fn get_table_name()->&'static str;
    fn get_schema()->TableSchemaModel;
}


pub struct TableSchemaModel{
    pub name: String,
    pub fields: Vec<ColumnSchemaModel>,
    pub indexes: Vec<IndexModel>
}

pub struct ColumnSchemaModel{
    pub name: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub sql_type: SqlType,
}
pub struct IndexModel{
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool
}

impl ColumnSchemaModel {
    pub fn new<T: ToSqlType>(name: String,is_primary_key: bool,is_nullable: bool,filed_type: T)->Self{
        Self{
            name,
            is_primary_key,
            is_nullable,
            sql_type: filed_type.to_sql_type(),
        }
    }
}

