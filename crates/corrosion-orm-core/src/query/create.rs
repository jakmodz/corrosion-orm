use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{query_type::QueryContext, to_sql::ToSql},
    schema::table::{SchemaValidationError, TableSchemaModel},
};

/// CREATE TABLE query builder.
///
/// Builds table creation SQL from [`TableSchemaModel`] using the active SQL dialect.
///
/// # Examples
///
/// ```
/// use corrosion_orm_core::query::create::Create;
/// use corrosion_orm_core::schema::table::TableSchemaModel;
///
/// let schema = TableSchemaModel::new("users".to_string());
/// let _create = Create::new(schema).if_not_exists();
/// ```
pub struct Create {
    schema: TableSchemaModel,
    if_not_exists: bool,
}

impl Create {
    pub fn new(schema: TableSchemaModel) -> Self {
        Self {
            schema,
            if_not_exists: false,
        }
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn build_sql(&self, dialect: &dyn SqlDialect) -> Result<String, SchemaValidationError> {
        if self.if_not_exists {
            dialect.generate_ddl_if_not_exists(&self.schema)
        } else {
            dialect.generate_ddl(&self.schema)
        }
    }
}

impl From<TableSchemaModel> for Create {
    fn from(schema: TableSchemaModel) -> Self {
        Self::new(schema)
    }
}

impl ToSql for Create {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        let sql = self.build_sql(dialect).unwrap_or_else(|err| {
            panic!(
                "failed to generate CREATE DDL for table '{}': {}",
                self.schema.name, err
            )
        });
        ctx.sql.push_str(&sql);
    }
}
