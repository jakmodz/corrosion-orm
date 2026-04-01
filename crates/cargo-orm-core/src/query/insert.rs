use std::borrow::Cow;

use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{query_type::Value, to_sql::ToSql},
    schema::table::TableSchemaModel,
};

#[derive(Debug, Clone)]
pub struct Insert<'query> {
    table: Cow<'query, str>,
    columns: Vec<Cow<'query, str>>,
    values: Vec<Value>,
}
impl<'query> Insert<'query> {
    pub fn new(table: &'query str) -> Self {
        Self {
            table: Cow::Borrowed(table),
            columns: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn columns(mut self, columns: Vec<Cow<'query, str>>) -> Self {
        self.columns = columns;
        self
    }
    pub fn values(mut self, values: Vec<Value>) -> Self {
        self.values = values;
        self
    }
}
impl<'col> From<&'col TableSchemaModel> for Insert<'col> {
    fn from(schema: &'col TableSchemaModel) -> Self {
        Insert {
            table: Cow::Borrowed(schema.name.as_str()),
            columns: schema
                .get_column_names()
                .into_iter()
                .map(Cow::Borrowed)
                .collect(),
            values: Vec::new(),
        }
    }
}
impl ToSql for Insert<'_> {
    fn to_sql(&self, ctx: &mut super::query_type::QueryContext, dialect: &dyn SqlDialect) {
        ctx.sql.push_str(&format!(
            "INSERT INTO {} ({}) VALUES(",
            self.table,
            self.columns.join(", "),
        ));
        for (i, value) in self.values.iter().enumerate() {
            if i > 0 {
                ctx.sql.push_str(", ");
            }
            ctx.push_bind_param(value.clone(), dialect);
        }
        ctx.sql.push(')');
    }
}
