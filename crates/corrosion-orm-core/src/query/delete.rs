use crate::{
    dialect::sql_dialect::SqlDialect,
    prelude::TableSchemaModel,
    query::{query_type::QueryContext, to_sql::ToSql, where_clause::WhereClause},
    types::ColumnTrait,
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
///
/// #[derive(Clone, Copy)]
/// pub enum UserColumn {
///     Id,
///     Name,
/// }
///
/// impl corrosion_orm_core::types::ColumnTrait for UserColumn {
///     fn as_str(&self) -> &'static str {
///         match self {
///             Self::Id => "id",
///             Self::Name => "name",
///         }
///     }
/// }
///
/// let delete = Delete::<UserColumn>::new("users");
/// ```
pub struct Delete<'query, C: ColumnTrait> {
    table: Cow<'query, str>,
    where_clause: Option<WhereClause<C>>,
}

impl<'query, C: ColumnTrait> Delete<'query, C> {
    pub fn new(table: &'query str) -> Self {
        Self {
            table: Cow::Borrowed(table),
            where_clause: None,
        }
    }

    pub fn where_clause(mut self, clause: WhereClause<C>) -> Self {
        self.where_clause = Some(clause);
        self
    }
}

impl<'query, C: ColumnTrait> ToSql for Delete<'query, C> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        ctx.sql.push_str(&format!("DELETE FROM {} ", self.table));
        if let Some(clause) = &self.where_clause {
            ctx.sql.push_str("WHERE ");
            clause.to_sql(ctx, dialect);
        }
    }
}
impl<'query, C: ColumnTrait> From<&'query TableSchemaModel> for Delete<'query, C> {
    fn from(table: &'query TableSchemaModel) -> Self {
        Self {
            table: Cow::Borrowed(table.name.as_str()),
            where_clause: None,
        }
    }
}
