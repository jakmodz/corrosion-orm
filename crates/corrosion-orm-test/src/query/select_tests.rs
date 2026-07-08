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
        /// Table name for this column.
        ///
        /// # Examples
        ///
        /// ```
        /// let c = Col("id");
        /// assert_eq!(c.table_name(), "users");
        /// ```
        fn table_name(&self) -> &'static str {
            "users"
        }

        /// Returns the underlying column name held by the `Col` wrapper.
        ///
        /// # Examples
        ///
        /// ```
        /// let c = Col("status");
        /// assert_eq!(c.column_name(), "status");
        /// ```
        fn column_name(&self) -> &'static str {
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
    /// Verifies that a SELECT with a simple equality WHERE clause renders using a table-qualified column.
    ///
    /// This test builds a `Select` selecting `id` and `name` from `users` with a `WHERE users.status = 'active'`
    /// condition and asserts the rendered SQL matches the expected snapshot.
    ///
    /// # Examples
    ///
    /// ```
    /// let where_clause = WhereClause {
    ///     clause: WhereClauseType::Condition(Condition::Eq(
    ///         Col("status"),
    ///         Value::String("active".to_string()),
    ///     )),
    /// };
    /// let select = Select::new("users")
    ///     .add_column("id")
    ///     .add_column("name")
    ///     .where_clause(where_clause);
    /// let sql = render_select(select);
    /// assert!(sql.contains("WHERE users.status = ?"));
    /// ```
    #[test]
    fn test_select_with_simple_where() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::Eq(
                Col("status").as_qualified(),
                Value::String("active".to_string()),
            )),
        };
        let select = Select::new("users")
            .add_column("id")
            .add_column("name")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT id, name FROM users WHERE users.status = ?");
    }

    #[test]
    fn test_select_with_where_and_limit() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::Gt(
                Col("age").as_qualified(),
                Value::Int(18),
            )),
        };
        let select = Select::new("users")
            .add_column("name")
            .where_clause(where_clause)
            .limit(5);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name FROM users WHERE users.age > ? LIMIT 5");
    }

    /// Verifies SQL generation for a SELECT with an `AND` where clause combining `=` and `>`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Builds a WHERE (users.status = ? AND users.score > ?) and renders SQL.
    /// let where_clause = WhereClause {
    ///     clause: WhereClauseType::And(
    ///         Box::new(WhereClauseType::Condition(Condition::Eq(Col("status"), Value::String("active".to_string())))),
    ///         Box::new(WhereClauseType::Condition(Condition::Gt(Col("score"), Value::Int(50))))),
    /// };
    /// let select = Select::new("users").add_column("name").where_clause(where_clause);
    /// let sql = render_select(select);
    /// assert!(sql.contains("users.status = ? AND users.score > ?"));
    /// ```
    #[test]
    fn test_select_with_and_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    Col("status").as_qualified(),
                    Value::String("active".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    Col("score").as_qualified(),
                    Value::Int(50),
                ))),
            ),
        };
        let select = Select::new("users")
            .add_column("name")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name FROM users WHERE users.status = ? AND users.score > ?");
    }

    #[test]
    fn test_select_with_or_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    Col("role").as_qualified(),
                    Value::String("admin".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    Col("role").as_qualified(),
                    Value::String("moderator".to_string()),
                ))),
            ),
        };
        let select = Select::new("users")
            .add_column("username")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT username FROM users WHERE users.role = ? OR users.role = ?");
    }

    #[test]
    fn test_select_with_in_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::In(
                Col("status").as_qualified(),
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
        insta::assert_snapshot!(sql, @"SELECT id, total FROM orders WHERE users.status IN (?, ?, ?)");
    }

    /// Verifies that a SELECT query with a LIKE condition renders a table-qualified column in SQL.
    ///
    /// Builds a `Select` with a `WHERE ... LIKE` clause on the `email` column and asserts the
    /// generated SQL contains `users.email LIKE ?`.
    ///
    /// # Examples
    ///
    /// ```
    /// let where_clause = WhereClause {
    ///     clause: WhereClauseType::Condition(Condition::Like(
    ///         Col("email"),
    ///         Value::String("%@gmail.com".to_string()),
    ///     )),
    /// };
    /// let select = Select::new("users")
    ///     .add_column("name")
    ///     .where_clause(where_clause);
    /// let sql = render_select(select);
    /// assert_eq!(sql, "SELECT name FROM users WHERE users.email LIKE ?");
    /// ```
    #[test]
    fn test_select_with_like_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::Like(
                Col("email").as_qualified(),
                Value::String("%@gmail.com".to_string()),
            )),
        };
        let select = Select::new("users")
            .add_column("name")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name FROM users WHERE users.email LIKE ?");
    }

    #[test]
    fn test_select_with_is_null() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Condition(Condition::IsNull(Col("deleted_at").as_qualified())),
        };
        let select = Select::new("posts")
            .add_column("title")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT title FROM posts WHERE users.deleted_at IS NULL");
    }

    /// Ensures a SELECT with a `NOT`-wrapped equality condition renders the expected SQL.
    ///
    /// # Examples
    ///
    /// ```
    /// let where_clause = WhereClause {
    ///     clause: WhereClauseType::Not(Box::new(WhereClauseType::Condition(Condition::Eq(
    ///         Col("banned"),
    ///         Value::Bool(true),
    ///     )))),
    /// };
    /// let select = Select::new("users")
    ///     .add_column("id")
    ///     .where_clause(where_clause);
    /// let sql = render_select(select);
    /// assert_eq!(sql, "SELECT id FROM users WHERE NOT users.banned = ?")
    /// ```
    #[test]
    fn test_select_with_not_condition() {
        let where_clause = WhereClause {
            clause: WhereClauseType::Not(Box::new(WhereClauseType::Condition(Condition::Eq(
                Col("banned").as_qualified(),
                Value::Bool(true),
            )))),
        };
        let select = Select::new("users")
            .add_column("id")
            .where_clause(where_clause);
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT id FROM users WHERE NOT users.banned = ?");
    }

    #[test]
    fn test_select_with_complex_where() {
        let where_clause = WhereClause {
            clause: WhereClauseType::And(
                Box::new(WhereClauseType::Or(
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        Col("role").as_qualified(),
                        Value::String("admin".to_string()),
                    ))),
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        Col("role").as_qualified(),
                        Value::String("moderator".to_string()),
                    ))),
                )),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    Col("experience").as_qualified(),
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
        insta::assert_snapshot!(sql, @"SELECT name, role FROM users WHERE (users.role = ? OR users.role = ?) AND users.experience > ? LIMIT 20");
    }
    #[test]
    fn test_select_with_order_by_asc() {
        let select = Select::new("users")
            .add_column("name")
            .add_column("role")
            .add_order_by(OrderBy::new(Col("name"), OrderDirection::Asc));
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name, role FROM users ORDER BY users.name ASC");
    }
    #[test]
    fn test_select_with_order_by_desc() {
        let select = Select::new("users")
            .add_column("name")
            .add_column("role")
            .add_order_by(OrderBy::new(Col("name"), OrderDirection::Desc));
        let sql = render_select(select);
        insta::assert_snapshot!(sql, @"SELECT name, role FROM users ORDER BY users.name DESC");
    }
}
