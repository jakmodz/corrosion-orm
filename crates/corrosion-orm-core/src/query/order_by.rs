use crate::{
    dialect::sql_dialect::SqlDialect, prelude::QueryContext, query::ToSql,
    types::column_trait::ColumnTrait,
};

#[derive(Debug, Clone)]
pub struct OrderClause<C: ColumnTrait> {
    pub columns: Vec<OrderBy<C>>,
}

impl<C: ColumnTrait> OrderClause<C> {
    pub fn new(columns: Vec<OrderBy<C>>) -> Self {
        Self { columns }
    }
}

impl<C: ColumnTrait> ToSql for OrderClause<C> {
    fn to_sql(&self, ctx: &mut QueryContext, _dialect: &dyn SqlDialect) {
        for (i, order) in self.columns.iter().enumerate() {
            if i > 0 {
                ctx.sql.push_str(", ");
            }
            order.to_sql(ctx, _dialect);
        }
    }
}
#[derive(Debug, Clone)]
pub struct OrderBy<C: ColumnTrait> {
    pub column: C,
    pub direction: OrderDirection,
}
#[derive(Debug, Clone)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl<C: ColumnTrait> OrderBy<C> {
    pub fn new(column: C, direction: OrderDirection) -> Self {
        Self { column, direction }
    }
}

impl<C: ColumnTrait> ToSql for OrderBy<C> {
    /// Appends an `ORDER BY` expression for this column and its direction to the query SQL.
    ///
    /// The column is rendered qualified into `ctx`, then the direction (`" ASC"` or `" DESC"`) is appended.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::{OrderBy, OrderDirection, QueryContext};
    ///
    /// let mut ctx = QueryContext::default();
    /// // `col` should be a value implementing the column trait used by your crate
    /// let col = /* construct a column, e.g. `users::id` */ todo!();
    /// let ob = OrderBy::new(col, OrderDirection::Desc);
    /// ob.to_sql(&mut ctx, &crate::dialect::DefaultSqlDialect);
    /// assert!(ctx.sql.ends_with(" DESC"));
    /// ```
    fn to_sql(&self, ctx: &mut QueryContext, _dialect: &dyn SqlDialect) {
        self.column.as_qualified().render(ctx);
        match self.direction {
            OrderDirection::Asc => ctx.sql.push_str(" ASC"),
            OrderDirection::Desc => ctx.sql.push_str(" DESC"),
        }
    }
}
