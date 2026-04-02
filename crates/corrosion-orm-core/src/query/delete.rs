use crate::{
    dialect::sql_dialect::SqlDialect,
    prelude::TableSchemaModel,
    query::{query_type::QueryContext, to_sql::ToSql, where_clause::WhereClause},
};
use std::borrow::Cow;
/// DELETE query builder.
///
/// Builds DELETE statements. WHERE clause is strongly recommended.
///
/// # Examples
///
/// ```
/// use corrosion_orm_core::query::delete::Delete;
/// let query = Delete::new("users");
/// ```
pub struct Delete<'query> {
    table: Cow<'query, str>,
    where_clause: Option<WhereClause<'query>>,
}

impl<'query> Delete<'query> {
    pub fn new(table: &'query str) -> Self {
        Self {
            table: Cow::Borrowed(table),
            where_clause: None,
        }
    }

    pub fn where_clause(mut self, clause: WhereClause<'query>) -> Self {
        self.where_clause = Some(clause);
        self
    }
}

impl<'query> ToSql for Delete<'query> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        ctx.sql.push_str(&format!("DELETE FROM {} ", self.table));
        if let Some(clause) = &self.where_clause {
            ctx.sql.push_str("WHERE ");
            clause.to_sql(ctx, dialect);
        }
    }
}
impl<'query> From<&'query TableSchemaModel> for Delete<'query> {
    fn from(table: &'query TableSchemaModel) -> Self {
        Self {
            table: Cow::Borrowed(table.name.as_str()),
            where_clause: None,
        }
    }
}
