use std::borrow::Cow;

use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{
        order_by::{OrderBy, OrderClause},
        to_sql::ToSql,
    },
    schema::table::TableSchemaModel,
    types::ColumnTrait,
};

use super::where_clause::WhereClause;

#[derive(Debug, Clone)]
/// SELECT query builder.
///
/// Builds SELECT queries with optional WHERE clauses and LIMIT.
///
/// The `C` generic type must implement `ColumnTrait` to guarantee type-safe
/// column references in the `WhereClause`. This replaces raw strings with
/// compile-time validated enum variants.
///
/// ```
/// use corrosion_orm_core::query::select::Select;
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
/// let select = Select::<UserColumn>::new("users");
/// ```
pub struct Select<'query, C: ColumnTrait> {
    table: Cow<'query, str>,
    columns: Vec<Cow<'query, str>>,
    where_clause: Option<WhereClause<C>>,
    order_by: Option<OrderClause<C>>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl<'col, C: ColumnTrait> Select<'col, C> {
    pub fn new<T: Into<Cow<'col, str>>>(table: T) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
        }
    }
    pub fn add_column<Column: Into<Cow<'col, str>>>(mut self, column: Column) -> Self {
        self.columns.push(column.into());
        self
    }
    pub fn columns<S: Into<Cow<'col, str>>>(mut self, columns: Vec<S>) -> Self {
        self.columns = columns.into_iter().map(|c| c.into()).collect();
        self
    }
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    pub fn where_clause(mut self, where_clause: WhereClause<C>) -> Self {
        self.where_clause = Some(where_clause);
        self
    }
    pub fn add_order_by(mut self, order_by: OrderBy<C>) -> Self {
        if let Some(order_clause) = &mut self.order_by {
            order_clause.columns.push(order_by);
        } else {
            self.order_by = Some(OrderClause {
                columns: vec![order_by],
            });
        }
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
    pub fn get_where_clause(&self) -> Option<&WhereClause<C>> {
        self.where_clause.as_ref()
    }
    #[cfg(feature = "test-utils")]
    pub fn get_limit(&self) -> Option<usize> {
        self.limit
    }
}
impl<C: ColumnTrait> ToSql for Select<'_, C> {
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
        if let Some(order_by) = &self.order_by {
            ctx.sql.push_str(" ORDER BY ");
            order_by.to_sql(ctx, _dialect);
        }
        if let Some(limit) = self.limit {
            ctx.sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            ctx.sql.push_str(&format!(" OFFSET {}", offset));
        }
    }
}
impl<'col, C: ColumnTrait> From<&'col TableSchemaModel> for Select<'col, C> {
    fn from(schema: &'col TableSchemaModel) -> Self {
        Self {
            table: Cow::Borrowed(&schema.name),
            columns: schema
                .get_column_names()
                .into_iter()
                .map(Cow::Borrowed)
                .collect(),
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
        }
    }
}
impl<'col, C: ColumnTrait> From<TableSchemaModel> for Select<'col, C> {
    fn from(schema: TableSchemaModel) -> Self {
        let mut columns = Vec::with_capacity(1 + schema.fields.len());

        columns.push(Cow::Owned(schema.primary_key.name));

        for field in schema.fields {
            columns.push(Cow::Owned(field.name));
        }

        Self {
            table: Cow::Owned(schema.name),
            columns,
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
        }
    }
}
