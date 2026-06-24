use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{ToSql, query_type::QueryContext},
    schema::table::TableSchemaModel,
};

pub struct Drop {
    pub table: String,
}
impl Drop {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
        }
    }
}
impl From<TableSchemaModel> for Drop {
    fn from(schema: TableSchemaModel) -> Self {
        Self::new(&schema.name)
    }
}

impl ToSql for Drop {
    fn to_sql(&self, ctx: &mut QueryContext, _dialect: &dyn SqlDialect) {
        ctx.sql.push_str(&format!("DROP TABLE {}", self.table));
    }
}
