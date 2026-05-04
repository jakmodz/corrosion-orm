use crate::prelude::QueryContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnRef {
    /// Just the column name: "id" (Used for INSERT, UPDATE)
    Bare(&'static str),
    /// Table and column name: "users.id" (Used for SELECT, JOIN, WHERE)
    Qualified(&'static str, &'static str),
}

impl ColumnRef {
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
