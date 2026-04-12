#[cfg(test)]
mod tests {
    use crate::User;
    use corrosion_orm_core::dialect::sqlite_dialect::sqlite::SqliteDialect;
    use corrosion_orm_core::query::order_by::{OrderBy, OrderDirection};
    use corrosion_orm_core::query::query_type::{QueryContext, Value};
    use corrosion_orm_core::query::select::Select;
    use corrosion_orm_core::query::to_sql::ToSql;
    use corrosion_orm_core::query::where_clause::{Condition, WhereClause, WhereClauseType};
    use corrosion_orm_core::schema::table::TableSchema;
    use corrosion_orm_core::types::ColumnTrait;

    #[derive(Clone, Copy, Debug)]
    pub struct Col(&'static str);
    impl ColumnTrait for Col {
        fn as_str(&self) -> &'static str {
            self.0
        }
    }

    fn render_select(select: Select<Col>) -> String {
        let mut ctx = QueryContext::new();
        select.to_sql(&mut ctx, &SqliteDialect);
        ctx.sql
    }
    #[test]
    fn test_select_create_from_table_schema_sqlite() {
        let mut ctx = QueryContext::new();
        let schema = User::get_schema();
        let select: Select<Col> = Select::from(&schema);
        select.to_sql(&mut ctx, &SqliteDialect);
        assert_eq!(select.get_columns().len(), 2);
        insta::assert_snapshot!(ctx.sql);
    }
    #[test]
    fn test_select_with_simple_where() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::Eq(
                Col("status"),
                Value::String("active".to_string()),
            )),
        };
        let select = Select::new("users")
            .add_column("id")
            .add_column("name")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT id, name FROM users WHERE status = ?");
    }

    #[test]
    fn test_select_with_where_and_limit() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::Gt(Col("age"), Value::Int(18))),
        };
        let select = Select::new("users")
            .add_column("name")
            .where_clause(where_clause)
            .limit(5);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name FROM users WHERE age > ? LIMIT 5");
    }

    #[test]
    fn test_select_with_and_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    Col("status"),
                    Value::String("active".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    Col("score"),
                    Value::Int(50),
                ))),
            ),
        };
        let select = Select::new("users")
            .add_column("name")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name FROM users WHERE status = ? AND score > ?");
    }

    #[test]
    fn test_select_with_or_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    Col("role"),
                    Value::String("admin".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    Col("role"),
                    Value::String("moderator".to_string()),
                ))),
            ),
        };
        let select = Select::new("users")
            .add_column("username")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT username FROM users WHERE role = ? OR role = ?");
    }

    #[test]
    fn test_select_with_in_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::In(
                Col("status"),
                vec![
                    Value::String("pending".to_string()),
                    Value::String("active".to_string()),
                    Value::String("approved".to_string()),
                ],
            )),
        };
        let select = Select::new("orders")
            .add_column("id")
            .add_column("total")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT id, total FROM orders WHERE status IN (?, ?, ?)");
    }

    #[test]
    fn test_select_with_like_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::Like(
                Col("email"),
                Value::String("%@gmail.com".to_string()),
            )),
        };
        let select = Select::new("users")
            .add_column("name")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name FROM users WHERE email LIKE ?");
    }

    #[test]
    fn test_select_with_is_null() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::IsNull(Col("deleted_at"))),
        };
        let select = Select::new("posts")
            .add_column("title")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT title FROM posts WHERE deleted_at IS NULL");
    }

    #[test]
    fn test_select_with_not_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Not(Box::new(WhereClauseType::Condition(Condition::Eq(
                Col("banned"),
                Value::Bool(true),
            )))),
        };
        let select = Select::new("users")
            .add_column("id")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT id FROM users WHERE NOT banned = ?");
    }

    #[test]
    fn test_select_with_complex_where() {
        let where_clause = WhereClause {
            clause: WhereClauseType::And(
                Box::new(WhereClauseType::Or(
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        Col("role"),
                        Value::String("admin".to_string()),
                    ))),
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        Col("role"),
                        Value::String("moderator".to_string()),
                    ))),
                )),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    Col("experience"),
                    Value::Int(5),
                ))),
            ),
        };
        let select = Select::new("users")
            .add_column("name")
            .add_column("role")
            .where_clause(where_clause)
            .limit(20);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name, role FROM users WHERE (role = ? OR role = ?) AND experience > ? LIMIT 20");
    }
    #[test]
    fn test_select_with_order_by_asc() {
        let select = Select::new("users")
            .add_column("name")
            .add_column("role")
            .add_order_by(OrderBy::new(Col("name"), OrderDirection::Asc));
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name, role FROM users ORDER BY name ASC");
    }
    #[test]
    fn test_select_with_order_by_desc() {
        let select = Select::new("users")
            .add_column("name")
            .add_column("role")
            .add_order_by(OrderBy::new(Col("name"), OrderDirection::Desc));
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name, role FROM users ORDER BY name DESC");
    }
}
