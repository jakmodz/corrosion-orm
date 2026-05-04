#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::test_entities::*;
    use corrosion_orm_core::{prelude::*, query::where_clause::WhereClause, types::ColumnTrait};

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

    fn render_update(update: Update<Col>) -> (String, Vec<Value>) {
        let mut ctx = QueryContext::new();
        update.to_sql(&mut ctx, &MockSqliteDialect);
        (ctx.sql, ctx.values)
    }

    #[test]
    fn test_update_single_column() {
        let update: Update<Col> = Update::new()
            .table(Cow::Owned("users".to_string()))
            .columns(vec![Cow::Owned("id".to_string())])
            .values(vec![Value::Int(1)]);
        let (sql, _values) = render_update(update);
        insta::assert_snapshot!(sql, @"UPDATE users SET id = ?");
    }
    #[test]
    fn test_update_multiple_columns() {
        let update: Update<Col> = Update::new()
            .table(Cow::Owned("users".to_string()))
            .columns(vec![
                Cow::Owned("id".to_string()),
                Cow::Owned("name".to_string()),
            ])
            .values(vec![Value::Int(1), Value::String("John".to_string())]);
        let (sql, _values) = render_update(update);
        insta::assert_snapshot!(sql, @"UPDATE users SET id = ?, name = ?");
    }
    #[test]
    fn test_update_with_where() {
        let schema = User::get_schema();
        let update = Update::from(&schema).where_clause(WhereClause::eq(Col("id"), 1));
        let (sql, _values) = render_update(update);
        insta::assert_snapshot!(sql, @"UPDATE users SET id = ?, name = ? WHERE users.id = ?");
    }
    #[test]
    fn test_update_with_where_and_multiple_columns() {
        let update = Update::new()
            .table(Cow::Owned("users".to_string()))
            .columns(vec![
                Cow::Owned("id".to_string()),
                Cow::Owned("name".to_string()),
            ])
            .values(vec![Value::Int(1), Value::String("John".to_string())])
            .where_clause(WhereClause::eq(Col("id"), 1));
        let (sql, _values) = render_update(update);
        insta::assert_snapshot!(sql, @"UPDATE users SET id = ?, name = ? WHERE users.id = ?");
    }
    #[test]
    fn test_update_with_where_and_multiple_columns_and_multiple_values() {
        let update = Update::new()
            .table(Cow::Owned("users".to_string()))
            .columns(vec![
                Cow::Owned("id".to_string()),
                Cow::Owned("name".to_string()),
            ])
            .values(vec![Value::Int(1), Value::String("John".to_string())])
            .where_clause(WhereClause::eq(Col("id"), 1));
        let (sql, _values) = render_update(update);
        insta::assert_snapshot!(sql, @"UPDATE users SET id = ?, name = ? WHERE users.id = ?");
    }
    #[test]
    fn test_update_from_user_schema() {
        let schema = User::get_schema();
        let update = Update::from(&schema).where_clause(WhereClause::eq(Col("id"), 1));
        let (sql, _values) = render_update(update);
        insta::assert_snapshot!(sql, @"UPDATE users SET id = ?, name = ? WHERE users.id = ?");
    }
}
