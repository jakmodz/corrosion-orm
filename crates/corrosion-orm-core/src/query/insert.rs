use std::borrow::Cow;

use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{query_type::Value, to_sql::ToSql},
    schema::table::TableSchemaModel,
};

#[derive(Debug, Clone)]
/// INSERT query builder.
///
/// Builds INSERT statements with automatic parameter binding.
/// Columns and values must be paired in order.
///
/// # Examples
///
/// ```
/// use corrosion_orm_core::query::insert::Insert;
/// let query = Insert::new("users")
///     .columns(vec!["name", "email"])
///     .values(vec![String::from("John"), String::from("john@example.com")]);
/// ```
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
    pub fn columns<S: Into<Cow<'query, str>>>(mut self, columns: Vec<S>) -> Self {
        self.columns = columns.into_iter().map(|c| c.into()).collect();
        self
    }
    pub fn values<V: Into<Value>>(mut self, values: Vec<V>) -> Self {
        self.values = values.into_iter().map(|v| v.into()).collect();
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
