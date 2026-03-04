use crate::{schema::table::{PrimaryKeyModel, TableSchemaModel}, types::column_type::SqlType};


static PRIMARY_KEY_TYPE: &'static str = "PRIMARY KEY";
static NOT_NULL: &'static str = "NOT NULL";
pub trait SqlDialect {
    
    fn cast_type(&self,sql_type: &SqlType) -> String;
    fn cast_primary_key(&self,priamry_key:&PrimaryKeyModel) -> String{
        format!("    {} {} {}\n", &priamry_key.name, 
            self.cast_type(&priamry_key.ty),
            PRIMARY_KEY_TYPE
        )
    }
    /// Generates the DDL for the given schema.
    /// * It is default implementation for dialect *
    fn generate_ddl(&self, schema: &TableSchemaModel) -> String {
        let mut ddl = format!("CREATE TABLE {} (\n", schema.name);
        ddl.push_str(self.cast_primary_key(&schema.primary_key).as_str());
        for field in schema.fields.iter() {
            ddl.push_str(&format!("    {} {}{}\n", field.name, 
                self.cast_type(&field.sql_type), 
                if field.is_nullable { "" } else { NOT_NULL })
            );
        }
        ddl.push_str(");\n");
        ddl
    }
}
