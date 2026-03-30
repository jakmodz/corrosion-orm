use std::borrow::Cow;

use crate::{
    dialect::sql_dialect::SqlDialect, query::to_sql::ToSql, schema::table::TableSchemaModel,
};

use super::where_clause::WhereClause;

#[derive(Debug, Clone)]
pub struct Select<'query> {
    table: Cow<'query, str>,
    columns: Vec<Cow<'query, str>>,
    where_clause: Option<WhereClause<'query>>,
    limit: Option<usize>,
}

impl<'col> Select<'col> {
    pub fn new<T: Into<Cow<'col, str>>>(table: T) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            where_clause: None,
            limit: None,
        }
    }
    pub fn add_column<C: Into<Cow<'col, str>>>(mut self, column: C) -> Self {
        self.columns.push(column.into());
        self
    }
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn where_clause(mut self, where_clause: WhereClause<'col>) -> Self {
        self.where_clause = Some(where_clause);
        self
    }
    #[cfg(feature = "test-utils")]
    pub fn get_table(&self) -> &str {
        &self.table
    }
    #[cfg(feature = "test-utils")]
    pub fn get_columns(&self) -> &[Cow<'col, str>] {
        &self.columns
    }
    #[cfg(feature = "test-utils")]
    pub fn get_where_clause(&self) -> Option<&WhereClause<'col>> {
        self.where_clause.as_ref()
    }
    #[cfg(feature = "test-utils")]
    pub fn get_limit(&self) -> Option<usize> {
        self.limit
    }
}
impl ToSql for Select<'_> {
    fn to_sql(&self, ctx: &mut super::query_type::QueryContext, _dialect: &dyn SqlDialect) {
        ctx.sql.push_str(&format!(
            "SELECT {} FROM {}",
            if self.columns.is_empty() {
                String::from("*")
            } else {
                self.columns.join(", ")
            },
            self.table
        ));
        if let Some(where_clause) = &self.where_clause {
            ctx.sql.push_str(" WHERE ");
            where_clause.to_sql(ctx, _dialect);
        }
        if let Some(limit) = self.limit {
            ctx.sql.push_str(&format!(" LIMIT {}", limit));
        }
    }
}
impl<'col> From<&'col TableSchemaModel> for Select<'col> {
    fn from(schema: &'col TableSchemaModel) -> Self {
        Self {
            table: Cow::Borrowed(&schema.name),
            columns: schema
                .get_column_names()
                .into_iter()
                .map(Cow::Borrowed)
                .collect(),
            where_clause: None,
            limit: None,
        }
    }
}

mod tests {

    #[test]
    fn test_select_create() {
        use super::*;
        let select = Select::new("users")
            .add_column("id")
            .add_column("name")
            .limit(10);
        assert_eq!(select.table, "users");
        assert_eq!(select.columns, &["id", "name"]);
        assert_eq!(select.where_clause, None);
        assert_eq!(select.limit, Some(10));
    }
}
