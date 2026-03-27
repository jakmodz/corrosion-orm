#[cfg(test)]
mod tests {
    use cargo_orm_core::dialect::sqlite_dialect::sqlite::SqliteDialect;
    use cargo_orm_core::query::insert::Insert;
    use cargo_orm_core::query::query_type::{QueryContext, Value};
    use cargo_orm_core::query::to_sql::ToSql;
    use std::borrow::Cow;

    fn render_insert(insert: Insert) -> (String, Vec<Value>) {
        let mut ctx = QueryContext::new();
        insert.to_sql(&mut ctx, &SqliteDialect);
        (ctx.sql, ctx.values)
    }

    #[test]
    fn test_insert_single_row() {
        let insert = Insert::new("users")
            .columns(vec![Cow::Borrowed("name"), Cow::Borrowed("age")])
            .values(vec![Value::String("Alice".to_string()), Value::Int(30)]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO users (name, age) VALUES(?, ?)");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_insert_single_column() {
        let insert = Insert::new("posts")
            .columns(vec![Cow::Borrowed("title")])
            .values(vec![Value::String("Hello World".to_string())]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO posts (title) VALUES(?)");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_insert_many_columns() {
        let insert = Insert::new("products")
            .columns(vec![
                Cow::Borrowed("name"),
                Cow::Borrowed("price"),
                Cow::Borrowed("stock"),
                Cow::Borrowed("active"),
            ])
            .values(vec![
                Value::String("Laptop".to_string()),
                Value::Int(999),
                Value::Int(50),
                Value::Bool(true),
            ]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO products (name, price, stock, active) VALUES(?, ?, ?, ?)");
        assert_eq!(values.len(), 4);
    }

    #[test]
    fn test_insert_all_value_types() {
        let insert = Insert::new("data")
            .columns(vec![
                Cow::Borrowed("text"),
                Cow::Borrowed("int_val"),
                Cow::Borrowed("int64_val"),
                Cow::Borrowed("bool_val"),
            ])
            .values(vec![
                Value::String("text".to_string()),
                Value::Int(42),
                Value::Int64(12345),
                Value::Bool(true),
            ]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO data (text, int_val, int64_val, bool_val) VALUES(?, ?, ?, ?)");
        assert_eq!(values.len(), 4);
    }

    #[test]
    fn test_insert_empty_string() {
        let insert = Insert::new("messages")
            .columns(vec![Cow::Borrowed("content")])
            .values(vec![Value::String("".to_string())]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO messages (content) VALUES(?)");
        assert_eq!(values[0], Value::String("".to_string()));
    }

    #[test]
    fn test_insert_special_chars() {
        let insert = Insert::new("posts")
            .columns(vec![Cow::Borrowed("title")])
            .values(vec![Value::String("Hello! @#$% \"quoted\"".to_string())]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO posts (title) VALUES(?)");
        assert_eq!(
            values[0],
            Value::String("Hello! @#$% \"quoted\"".to_string())
        );
    }

    #[test]
    fn test_insert_negative_number() {
        let insert = Insert::new("accounts")
            .columns(vec![Cow::Borrowed("balance")])
            .values(vec![Value::Int(-100)]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO accounts (balance) VALUES(?)");
        assert_eq!(values[0], Value::Int(-100));
    }

    #[test]
    fn test_insert_with_underscores() {
        let insert = Insert::new("user_profiles")
            .columns(vec![
                Cow::Borrowed("first_name"),
                Cow::Borrowed("last_name"),
            ])
            .values(vec![
                Value::String("John".to_string()),
                Value::String("Doe".to_string()),
            ]);
        let (sql, _) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO user_profiles (first_name, last_name) VALUES(?, ?)");
    }

    #[test]
    fn test_insert_no_values() {
        let insert = Insert::new("users")
            .columns(vec![Cow::Borrowed("name")])
            .values(vec![]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO users (name) VALUES()");
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_insert_builder_overwrite() {
        let insert = Insert::new("users")
            .columns(vec![Cow::Borrowed("old")])
            .columns(vec![Cow::Borrowed("new")])
            .values(vec![Value::String("value".to_string())]);
        let (sql, _) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO users (new) VALUES(?)");
    }

    #[test]
    fn test_insert_real_world_user() {
        let insert = Insert::new("users")
            .columns(vec![
                Cow::Borrowed("username"),
                Cow::Borrowed("email"),
                Cow::Borrowed("age"),
                Cow::Borrowed("verified"),
            ])
            .values(vec![
                Value::String("john_doe".to_string()),
                Value::String("john@example.com".to_string()),
                Value::Int(28),
                Value::Bool(false),
            ]);
        let (sql, values) = render_insert(insert);
        insta::assert_snapshot!(sql, @"INSERT INTO users (username, email, age, verified) VALUES(?, ?, ?, ?)");
        assert_eq!(values.len(), 4);
    }
}
