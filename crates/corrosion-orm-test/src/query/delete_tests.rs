#[cfg(test)]
mod tests {
    use crate::{MockSqliteDialect, User};
    use corrosion_orm_core::{
        prelude::{Delete, QueryContext, TableSchema, ToSql, Value},
        query::where_clause::WhereClause,
        types::ColumnTrait,
    };

    #[derive(Clone, Copy, Debug)]
    pub struct Col(&'static str);
    impl ColumnTrait for Col {
        fn table_name(&self) -> &'static str {
            "users"
        }

        fn column_name(&self) -> &'static str {
            self.0
        }
    }

    fn render_delete(delete: Delete<Col>) -> (String, Vec<Value>) {
        let mut ctx = QueryContext::new();
        delete.to_sql(&mut ctx, &MockSqliteDialect);
        (ctx.sql, ctx.values)
    }

    #[test]
    fn test_delete_simple() {
        let delete = Delete::new("users").where_clause(WhereClause::eq(Col("id"), 1));
        let (sql, values) = render_delete(delete);
        insta::assert_snapshot!(sql, @"DELETE FROM users WHERE users.id = ?");
        assert_eq!(values.len(), 1)
    }
    #[test]
    fn test_delete_complex() {
        let delete = Delete::new("users").where_clause(WhereClause::and(
            WhereClause::eq(Col("id"), 1),
            WhereClause::eq(Col("name"), String::from("John")),
        ));
        let (sql, values) = render_delete(delete);
        insta::assert_snapshot!(sql, @"DELETE FROM users WHERE users.id = ? AND users.name = ?");
        assert_eq!(values.len(), 2)
    }
    #[test]
    fn test_from_schema() {
        let schema = User::get_schema();
        let delete = Delete::from(&schema).where_clause(WhereClause::eq(Col("id"), 1));
        let (sql, values) = render_delete(delete);
        insta::assert_snapshot!(sql, @"DELETE FROM users WHERE users.id = ?");
        assert_eq!(values.len(), 1)
    }
}
