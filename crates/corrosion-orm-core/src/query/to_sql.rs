use crate::{dialect::sql_dialect::SqlDialect, query::query_type::QueryContext};
/// Convert query builders to SQL strings.
///
/// This trait is automatically implemented for all query builders.
/// You typically don't need to implement this yourself.
///
/// # Examples
///
/// ```
/// use corrosion_orm_core::query::to_sql::ToSql;
/// use corrosion_orm_core::query::query_type::QueryContext;
/// // let mut ctx = QueryContext::new();
/// // query.to_sql(&mut ctx, &dialect);
/// ```
pub trait ToSql {
    /// Converts entity to SQL. Generated SQL is pushed into the query context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The query context to populate with the SQL and values.
    /// * `dialect` - The SQL dialect to use for generating the SQL.
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect);
}
