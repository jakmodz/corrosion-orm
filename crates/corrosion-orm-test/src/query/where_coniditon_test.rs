#[cfg(test)]
mod tests {

    use crate::test_entities::MockSqliteDialect;
    use crate::test_entities::*;
    use corrosion_orm_core::query::query_type::{QueryContext, Value};
    use corrosion_orm_core::query::to_sql::ToSql;
    use corrosion_orm_core::query::where_clause::{Condition, WhereClause, WhereClauseType};
    fn render_clause(clause: WhereClauseType) -> (String, Vec<Value>) {
        let mut ctx = QueryContext::new();
        let dialect = MockSqliteDialect;
        clause.to_sql(&mut ctx, &dialect);
        (ctx.sql, ctx.values)
    }

    #[test]
    fn test_simple_eq() {
        let clause = WhereClauseType::Condition(Condition::Eq("age", Value::Int(18)));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"age = ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_ne() {
        let clause = WhereClauseType::Condition(Condition::Ne(
            "status",
            Value::String("inactive".to_string()),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"status != ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_lt() {
        let clause = WhereClauseType::Condition(Condition::Lt("price", Value::Int(100)));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"price < ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_gt() {
        let clause = WhereClauseType::Condition(Condition::Gt("score", Value::Int(50)));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"score > ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_lte() {
        let clause = WhereClauseType::Condition(Condition::Lte("age", Value::Int(65)));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"age <= ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_gte() {
        let clause = WhereClauseType::Condition(Condition::Gte("rating", Value::Int(4)));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"rating >= ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_like() {
        let clause = WhereClauseType::Condition(Condition::Like(
            "email",
            Value::String("%@gmail.com".to_string()),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"email LIKE ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_not_like() {
        let clause = WhereClauseType::Condition(Condition::NotLike(
            "name",
            Value::String("test%".to_string()),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"name NOT LIKE ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_in() {
        let clause = WhereClauseType::Condition(Condition::In(
            "status",
            vec![
                Value::String("active".to_string()),
                Value::String("pending".to_string()),
            ],
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"status IN (?, ?)");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_simple_not_in() {
        let clause = WhereClauseType::Condition(Condition::NotIn(
            "category",
            vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"category NOT IN (?, ?, ?)");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_simple_is_null() {
        let clause = WhereClauseType::Condition(Condition::IsNull("deleted_at"));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"deleted_at IS NULL");
        assert_eq!(values.len(), 0);
    }
    #[test]
    fn test_simple_between() {
        let clause =
            WhereClauseType::Condition(Condition::Between("age", Value::Int(18), Value::Int(30)));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"age BETWEEN ? AND ?");
        assert_eq!(values.len(), 2);
    }
    #[test]
    fn test_and_two_conditions() {
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "age",
                Value::Int(18),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "status",
                Value::String("active".to_string()),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"age = ? AND status = ?");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_and_three_conditions() {
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "age",
                Value::Int(18),
            ))),
            Box::new(WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "status",
                    Value::String("active".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    "score",
                    Value::Int(50),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"age = ? AND status = ? AND score > ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_or_two_conditions() {
        let clause = WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "role",
                Value::String("admin".to_string()),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "role",
                Value::String("moderator".to_string()),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"role = ? OR role = ?");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_or_three_conditions() {
        let clause = WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "status",
                Value::String("pending".to_string()),
            ))),
            Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "status",
                    Value::String("review".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "status",
                    Value::String("approved".to_string()),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"status = ? OR status = ? OR status = ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_and_with_or_needs_parens() {
        // (a OR b) AND c
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "age",
                    Value::Int(18),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "age",
                    Value::Int(21),
                ))),
            )),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "status",
                Value::String("active".to_string()),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"(age = ? OR age = ?) AND status = ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_and_with_or_right_needs_parens() {
        // a AND (b OR c)
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "verified",
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    "score",
                    Value::Int(80),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "premium",
                    Value::Bool(true),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"verified = ? AND (score > ? OR premium = ?)");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_or_with_and_no_extra_parens() {
        // a OR (b AND c) - AND has higher precedence, so no extra parens needed
        let clause = WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "admin",
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "moderator",
                    Value::Bool(true),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    "experience",
                    Value::Int(5),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"admin = ? OR moderator = ? AND experience > ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_complex_nested_expression() {
        // (a OR b) AND (c OR d) AND e
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::And(
                Box::new(WhereClauseType::Or(
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        "status",
                        Value::String("active".to_string()),
                    ))),
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        "status",
                        Value::String("pending".to_string()),
                    ))),
                )),
                Box::new(WhereClauseType::Or(
                    Box::new(WhereClauseType::Condition(Condition::Gt(
                        "age",
                        Value::Int(18),
                    ))),
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        "premium",
                        Value::Bool(true),
                    ))),
                )),
            )),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "deleted",
                Value::Bool(false),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"(status = ? OR status = ?) AND (age > ? OR premium = ?) AND deleted = ?");
        assert_eq!(values.len(), 5);
    }

    #[test]
    fn test_not_simple_condition() {
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::Condition(Condition::IsNull(
            "email",
        ))));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT email IS NULL");
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_not_and_condition() {
        // NOT (a AND b)
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "banned",
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Gt(
                "violations",
                Value::Int(5),
            ))),
        )));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT (banned = ? AND violations > ?)");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_not_or_condition() {
        // NOT (a OR b)
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "status",
                Value::String("deleted".to_string()),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "archived",
                Value::Bool(true),
            ))),
        )));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT (status = ? OR archived = ?)");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_not_not() {
        // NOT NOT a (double negation)
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::Not(Box::new(
            WhereClauseType::Condition(Condition::Eq("active", Value::Bool(true))),
        ))));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT NOT active = ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_not_in_and_expression() {
        // a AND NOT (b OR c)
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                "verified",
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::Not(Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "suspended",
                    Value::Bool(true),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "banned",
                    Value::Bool(true),
                ))),
            )))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"verified = ? AND NOT (suspended = ? OR banned = ?)");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_full_where_clause_wrapper() {
        let clause = WhereClause {
            clause: WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "age",
                    Value::Int(18),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    "active",
                    Value::Bool(true),
                ))),
            ),
        };
        let mut ctx = QueryContext::new();
        let dialect = MockSqliteDialect;
        clause.to_sql(&mut ctx, &dialect);
        insta::assert_snapshot!(ctx.sql, @"age = ? AND active = ?");
    }
    #[test]
    fn test_simple_condition_from_entity_column() {
        let clause: WhereClause<'_> = user::COLUMN.name.eq(Value::String("John".to_string()));
        let (sql, values) = render_clause(clause.clause);
        insta::assert_snapshot!(sql, @"name = ?");
        assert_eq!(values.len(), 1);
    }
    #[test]
    fn test_contains_from_entity_column() {
        let clause: WhereClause<'_> = user::COLUMN
            .name
            .contains(Value::String("John".to_string()));
        let (sql, values) = render_clause(clause.clause);

        insta::assert_snapshot!(sql, @"name LIKE ?");
        assert_eq!(values.len(), 1);
    }
    #[test]
    fn test_starts_with_entity_column() {
        let clause: WhereClause<'_> = user::COLUMN
            .name
            .contains(Value::String("John".to_string()));
        let (sql, values) = render_clause(clause.clause);
        insta::assert_snapshot!(sql, @"name LIKE ?");
        assert_eq!(values.len(), 1);
    }
    #[test]
    fn test_numeric_column_entity() {
        let clause: WhereClause<'_> = user::COLUMN.id.eq(30);
        let (sql, values) = render_clause(clause.clause);
        insta::assert_snapshot!(sql, @"id = ?");
        assert_eq!(values.len(), 1);
    }
}
