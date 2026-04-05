use crate::{
    schema::table::{
        ColumnSchemaModel, IndexModel, PrimaryKeyModel, SchemaValidationError, TableSchemaModel,
    },
    types::column_type::SqlType,
};

static PRIMARY_KEY_TYPE: &str = "PRIMARY KEY";
static TAB: &str = "    ";

/// Trait representing a SQL dialect.
/// Implement `cast_type` to support a new database engine; all DDL methods have
/// working defaults built on top of it.
pub trait SqlDialect: Send + Sync {
    /// Maps an [`SqlType`] to its database-specific type name.
    fn cast_type(&self, sql_type: &SqlType) -> String;

    /// Formats the PRIMARY KEY column definition (no trailing newline or comma).
    fn cast_primary_key(&self, primary_key: &PrimaryKeyModel) -> String {
        format!(
            "{}{} {} {}",
            TAB,
            primary_key.name,
            self.cast_type(&primary_key.ty),
            PRIMARY_KEY_TYPE
        )
    }

    /// Formats a single non-PK column definition (no trailing newline or comma).
    fn cast_column(&self, column: &ColumnSchemaModel) -> String {
        let mut s = format!(
            "{}{} {}",
            TAB,
            column.name,
            self.cast_type(&column.sql_type)
        );
        if column.is_nullable {
            s.push_str(" NULL");
        } else {
            s.push_str(" NOT NULL");
        }
        if column.is_unique {
            s.push_str(" UNIQUE");
        }
        s
    }
    /// Generates `CREATE TABLE … (…);`.
    /// Validates the schema first — returns [`SchemaValidationError`] if invalid.
    fn generate_ddl(&self, schema: &TableSchemaModel) -> Result<String, SchemaValidationError> {
        self.build_create_table_ddl(schema, false)
    }

    fn generate_ddl_if_not_exists(
        &self,
        schema: &TableSchemaModel,
    ) -> Result<String, SchemaValidationError> {
        self.build_create_table_ddl(schema, true)
    }

    fn build_create_table_ddl(
        &self,
        schema: &TableSchemaModel,
        if_not_exists: bool,
    ) -> Result<String, SchemaValidationError> {
        schema.validate()?;
        let guard = if if_not_exists { "IF NOT EXISTS " } else { "" };

        let mut columns: Vec<String> = Vec::with_capacity(1 + schema.fields.len());
        columns.push(self.cast_primary_key(&schema.primary_key));
        for field in &schema.fields {
            columns.push(self.cast_column(field));
        }
        let mut ddl = format!(
            "CREATE TABLE {}{} (\n{}\n);\n",
            guard,
            schema.name,
            columns.join(",\n")
        );
        for index in &schema.indexes {
            ddl.push_str(&self.generate_create_index_ddl(&schema.name, index));
        }

        Ok(ddl)
    }

    /// Generates `DROP TABLE <name>;`.
    fn generate_drop_table_ddl(&self, table_name: &str) -> String {
        format!("DROP TABLE {};\n", table_name)
    }

    /// Generates `DROP TABLE IF EXISTS <name>;`.
    fn generate_drop_table_if_exists_ddl(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS {};\n", table_name)
    }

    /// Generates `CREATE [UNIQUE] INDEX IF NOT EXISTS <name> ON <table> (<cols>);`.
    fn generate_create_index_ddl(&self, table_name: &str, index: &IndexModel) -> String {
        let unique = if index.unique { "UNIQUE " } else { "" };
        let columns = index.fields.join(", ");
        format!(
            "CREATE {}INDEX IF NOT EXISTS {} ON {} ({});\n",
            unique, index.name, table_name, columns
        )
    }

    /// Generates the full DDL for a table:
    /// `CREATE TABLE IF NOT EXISTS` + one `CREATE INDEX` per entry in `schema.indexes`.
    fn generate_full_ddl(
        &self,
        schema: &TableSchemaModel,
    ) -> Result<String, SchemaValidationError> {
        let mut ddl = self.generate_ddl_if_not_exists(schema)?;
        for index in &schema.indexes {
            ddl.push_str(&self.generate_create_index_ddl(&schema.name, index));
        }
        Ok(ddl)
    }

    fn bind_param(&self, count: &usize) -> String;
}
