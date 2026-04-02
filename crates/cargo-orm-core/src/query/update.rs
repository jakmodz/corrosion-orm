use crate::{
    dialect::sql_dialect::SqlDialect,
    prelude::TableSchemaModel,
    query::{
        query_type::{QueryContext, Value},
        to_sql::ToSql,
        where_clause::WhereClause,
    },
};
use std::borrow::Cow;
/// UPDATE query builder.
///
/// Builds UPDATE statements with automatic parameter binding.
/// WHERE clause is recommended to prevent unintended updates.
///
/// # Examples
///
/// ```
/// use cargo_orm_core::query::update::Update;
/// use std::borrow::Cow;
/// let query = Update::new()
///     .table(Cow::Borrowed("users"))
///     .columns(vec!["name"])
///     .table(std::borrow::Cow::Borrowed("users"));
/// ```
pub struct Update<'query> {
    table: Cow<'query, str>,
    columns: Vec<Cow<'query, str>>,
    values: Vec<Value>,
    where_clause: Option<WhereClause<'query>>,
}
impl<'query> Update<'query> {
    pub fn new() -> Self {
        Self {
            table: Cow::Owned(String::new()),
            columns: Vec::new(),
            values: Vec::new(),
            where_clause: None,
        }
    }
    pub fn table(mut self, table: Cow<'query, str>) -> Self {
        self.table = table;
        self
    }
    pub fn where_clause(mut self, where_clause: WhereClause<'query>) -> Self {
        self.where_clause = Some(where_clause);
        self
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
impl<'query> Default for Update<'query> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'query> ToSql for Update<'query> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        ctx.sql
            .push_str(&format!("UPDATE {} SET ", self.table.as_ref()));
        let pairs: Vec<_> = self.columns.iter().zip(self.values.iter()).collect();
        for (i, (column, value)) in pairs.iter().enumerate() {
            ctx.sql.push_str(&format!("{} = ", column.as_ref()));
            ctx.push_bind_param((*value).clone(), dialect);
            if i + 1 < pairs.len() {
                ctx.sql.push_str(", ");
            }
        }

        if let Some(where_clause) = &self.where_clause {
            ctx.sql.push_str(" WHERE ");
            where_clause.to_sql(ctx, dialect);
        }
    }
}

impl<'query> From<&'query TableSchemaModel> for Update<'query> {
    fn from(schema: &'query TableSchemaModel) -> Self {
        Update {
            table: Cow::Borrowed(&schema.name),
            columns: schema
                .get_column_names()
                .into_iter()
                .map(Cow::Borrowed)
                .collect(),
            values: vec![],
            where_clause: None,
        }
    }
}
