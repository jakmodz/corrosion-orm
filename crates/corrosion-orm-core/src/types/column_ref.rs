use crate::prelude::QueryContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnRef {
    /// Just the column name: "id" (Used for INSERT, UPDATE)
    Bare(&'static str),
    /// Table and column name: "users.id" (Used for SELECT, JOIN, WHERE)
    Qualified(&'static str, &'static str),
}

impl ColumnRef {
    /// Appends this column reference's SQL text to the provided query context's SQL buffer.
    ///
    /// - `Bare(column)` appends the column name.
    /// - `Qualified(table, column)` appends `table.column`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use corrosion_orm_core::prelude::QueryContext;
    /// use corrosion_orm_core::types::column_ref::ColumnRef;
    ///
    /// let mut ctx = QueryContext::default();
    /// ColumnRef::Bare("id").render(&mut ctx);
    /// assert_eq!(ctx.sql, "id");
    ///
    /// let mut ctx = QueryContext::default();
    /// ColumnRef::Qualified("users", "id").render(&mut ctx);
    /// assert_eq!(ctx.sql, "users.id");
    /// ```
    pub fn render(&self, ctx: &mut QueryContext) {
        match self {
            ColumnRef::Bare(col) => {
                ctx.sql.push_str(col);
            }
            ColumnRef::Qualified(table, col) => {
                ctx.sql.push_str(table);
                ctx.sql.push('.');
                ctx.sql.push_str(col);
            }
        }
    }
}
