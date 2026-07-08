#[cfg(test)]
mod tests {

    use crate::test_entities::*;
    use corrosion_orm_core::query::query_type::{QueryContext, Value};
    use corrosion_orm_core::query::to_sql::ToSql;
    use corrosion_orm_core::query::where_clause::{Condition, WhereClause, WhereClauseType};
    use corrosion_orm_core::types::ColumnTrait;

    #[derive(Clone, Copy)]
    pub enum TestColumn {
        Age,
        Status,
        Price,
        Score,
        Rating,
        Email,
        Name,
        Category,
        DeletedAt,
        Role,
        Verified,
        Premium,
        Admin,
        Moderator,
        Experience,
        Deleted,
        Banned,
        Violations,
        Archived,
        Active,
        Suspended,
    }

    impl ColumnTrait for TestColumn {
        fn table_name(&self) -> &'static str {
            "users"
        }

        fn column_name(&self) -> &'static str {
            match self {
                Self::Age => "age",
                Self::Status => "status",
                Self::Price => "price",
                Self::Score => "score",
                Self::Rating => "rating",
                Self::Email => "email",
                Self::Name => "name",
                Self::Category => "category",
                Self::DeletedAt => "deleted_at",
                Self::Role => "role",
                Self::Verified => "verified",
                Self::Premium => "premium",
                Self::Admin => "admin",
                Self::Moderator => "moderator",
                Self::Experience => "experience",
                Self::Deleted => "deleted",
                Self::Banned => "banned",
                Self::Violations => "violations",
                Self::Archived => "archived",
                Self::Active => "active",
                Self::Suspended => "suspended",
            }
        }
    }

    fn render_clause(clause: WhereClauseType) -> (String, Vec<Value>) {
        let mut ctx = QueryContext::new();
        let dialect = MockSqliteDialect;
        clause.to_sql(&mut ctx, &dialect);
        (ctx.sql, ctx.values)
    }

    #[test]
    fn test_simple_eq() {
        let clause = WhereClauseType::Condition(Condition::Eq(
            TestColumn::Age.as_qualified(),
            Value::Int(18),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.age = ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_ne() {
        let clause = WhereClauseType::Condition(Condition::Ne(
            TestColumn::Status.as_qualified(),
            Value::String("inactive".to_string()),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.status != ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_lt() {
        let clause = WhereClauseType::Condition(Condition::Lt(
            TestColumn::Price.as_qualified(),
            Value::Int(100),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.price < ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_gt() {
        let clause = WhereClauseType::Condition(Condition::Gt(
            TestColumn::Score.as_qualified(),
            Value::Int(50),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.score > ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_lte() {
        let clause = WhereClauseType::Condition(Condition::Lte(
            TestColumn::Age.as_qualified(),
            Value::Int(65),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.age <= ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_gte() {
        let clause = WhereClauseType::Condition(Condition::Gte(
            TestColumn::Rating.as_qualified(),
            Value::Int(4),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.rating >= ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_like() {
        let clause = WhereClauseType::Condition(Condition::Like(
            TestColumn::Email.as_qualified(),
            Value::String("%@gmail.com".to_string()),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.email LIKE ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_not_like() {
        let clause = WhereClauseType::Condition(Condition::NotLike(
            TestColumn::Name.as_qualified(),
            Value::String("test%".to_string()),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.name NOT LIKE ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_simple_in() {
        let clause = WhereClauseType::Condition(Condition::In(
            TestColumn::Status.as_qualified(),
            vec![
                Value::String("active".to_string()),
                Value::String("pending".to_string()),
            ],
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.status IN (?, ?)");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_simple_not_in() {
        let clause = WhereClauseType::Condition(Condition::NotIn(
            TestColumn::Category.as_qualified(),
            vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.category NOT IN (?, ?, ?)");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_simple_is_null() {
        let clause =
            WhereClauseType::Condition(Condition::IsNull(TestColumn::DeletedAt.as_qualified()));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.deleted_at IS NULL");
        assert_eq!(values.len(), 0);
    }
    #[test]
    fn test_simple_between() {
        let clause = WhereClauseType::Condition(Condition::Between(
            TestColumn::Age.as_qualified(),
            Value::Int(18),
            Value::Int(30),
        ));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.age BETWEEN ? AND ?");
        assert_eq!(values.len(), 2);
    }
    #[test]
    fn test_and_two_conditions() {
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Age.as_qualified(),
                Value::Int(18),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Status.as_qualified(),
                Value::String("active".to_string()),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.age = ? AND users.status = ?");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_and_three_conditions() {
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Age.as_qualified(),
                Value::Int(18),
            ))),
            Box::new(WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Status.as_qualified(),
                    Value::String("active".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    TestColumn::Score.as_qualified(),
                    Value::Int(50),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.age = ? AND users.status = ? AND users.score > ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_or_two_conditions() {
        let clause = WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Role.as_qualified(),
                Value::String("admin".to_string()),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Role.as_qualified(),
                Value::String("moderator".to_string()),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.role = ? OR users.role = ?");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_or_three_conditions() {
        let clause = WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Status.as_qualified(),
                Value::String("pending".to_string()),
            ))),
            Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Status.as_qualified(),
                    Value::String("review".to_string()),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Status.as_qualified(),
                    Value::String("approved".to_string()),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.status = ? OR users.status = ? OR users.status = ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_and_with_or_needs_parens() {
        // (a OR b) AND c
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Age.as_qualified(),
                    Value::Int(18),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Age.as_qualified(),
                    Value::Int(21),
                ))),
            )),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Status.as_qualified(),
                Value::String("active".to_string()),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"(users.age = ? OR users.age = ?) AND users.status = ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_and_with_or_right_needs_parens() {
        // a AND (b OR c)
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Verified.as_qualified(),
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    TestColumn::Score.as_qualified(),
                    Value::Int(80),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Premium.as_qualified(),
                    Value::Bool(true),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.verified = ? AND (users.score > ? OR users.premium = ?)");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_or_with_and_no_extra_parens() {
        // a OR (b AND c) - AND has higher precedence, so no extra parens needed
        let clause = WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Admin.as_qualified(),
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Moderator.as_qualified(),
                    Value::Bool(true),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Gt(
                    TestColumn::Experience.as_qualified(),
                    Value::Int(5),
                ))),
            )),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.admin = ? OR users.moderator = ? AND users.experience > ?");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_complex_nested_expression() {
        // (a OR b) AND (c OR d) AND e
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::And(
                Box::new(WhereClauseType::Or(
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        TestColumn::Status.as_qualified(),
                        Value::String("active".to_string()),
                    ))),
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        TestColumn::Status.as_qualified(),
                        Value::String("pending".to_string()),
                    ))),
                )),
                Box::new(WhereClauseType::Or(
                    Box::new(WhereClauseType::Condition(Condition::Gt(
                        TestColumn::Age.as_qualified(),
                        Value::Int(18),
                    ))),
                    Box::new(WhereClauseType::Condition(Condition::Eq(
                        TestColumn::Premium.as_qualified(),
                        Value::Bool(true),
                    ))),
                )),
            )),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Deleted.as_qualified(),
                Value::Bool(false),
            ))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"(users.status = ? OR users.status = ?) AND (users.age > ? OR users.premium = ?) AND users.deleted = ?");
        assert_eq!(values.len(), 5);
    }

    #[test]
    fn test_not_simple_condition() {
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::Condition(Condition::IsNull(
            TestColumn::Email.as_qualified(),
        ))));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT users.email IS NULL");
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_not_and_condition() {
        // NOT (a AND b)
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Banned.as_qualified(),
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Gt(
                TestColumn::Violations.as_qualified(),
                Value::Int(5),
            ))),
        )));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT (users.banned = ? AND users.violations > ?)");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_not_or_condition() {
        // NOT (a OR b)
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::Or(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Status.as_qualified(),
                Value::String("deleted".to_string()),
            ))),
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Archived.as_qualified(),
                Value::Bool(true),
            ))),
        )));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT (users.status = ? OR users.archived = ?)");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_not_not() {
        // NOT NOT a (double negation)
        let clause = WhereClauseType::Not(Box::new(WhereClauseType::Not(Box::new(
            WhereClauseType::Condition(Condition::Eq(
                TestColumn::Active.as_qualified(),
                Value::Bool(true),
            )),
        ))));
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"NOT NOT users.active = ?");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_not_in_and_expression() {
        // a AND NOT (b OR c)
        let clause = WhereClauseType::And(
            Box::new(WhereClauseType::Condition(Condition::Eq(
                TestColumn::Verified.as_qualified(),
                Value::Bool(true),
            ))),
            Box::new(WhereClauseType::Not(Box::new(WhereClauseType::Or(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Suspended.as_qualified(),
                    Value::Bool(true),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Banned.as_qualified(),
                    Value::Bool(true),
                ))),
            )))),
        );
        let (sql, values) = render_clause(clause);
        insta::assert_snapshot!(sql, @"users.verified = ? AND NOT (users.suspended = ? OR users.banned = ?)");
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_full_where_clause_wrapper() {
        let clause = WhereClause {
            clause: WhereClauseType::And(
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Age.as_qualified(),
                    Value::Int(18),
                ))),
                Box::new(WhereClauseType::Condition(Condition::Eq(
                    TestColumn::Active.as_qualified(),
                    Value::Bool(true),
                ))),
            ),
        };
        let mut ctx = QueryContext::new();
        let dialect = MockSqliteDialect;
        clause.to_sql(&mut ctx, &dialect);
        insta::assert_snapshot!(ctx.sql, @"users.age = ? AND users.active = ?");
    }
    #[test]
    fn test_simple_condition_from_entity_column() {
        let clause: WhereClause = user::COLUMN.name.eq(Value::String("John".to_string()));
        let (sql, values) = render_clause(clause.clause);
        insta::assert_snapshot!(sql, @"users.name = ?");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], Value::String("John".to_string()))
    }
    #[test]
    fn test_contains_from_entity_column() {
        let clause: WhereClause = user::COLUMN
            .name
            .contains(Value::String("John".to_string()));
        let (sql, values) = render_clause(clause.clause);

        insta::assert_snapshot!(sql, @"users.name LIKE ?");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], Value::String("%John%".to_string()));
    }
    #[test]
    fn test_starts_with_entity_column() {
        let clause: WhereClause = user::COLUMN
            .name
            .starts_with(Value::String("John".to_string()));
        let (sql, values) = render_clause(clause.clause);
        insta::assert_snapshot!(sql, @"users.name LIKE ?");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], Value::String("John%".to_string()));
    }
    #[test]
    fn test_numeric_column_entity() {
        let clause: WhereClause = user::COLUMN.id.eq(30);
        let (sql, values) = render_clause(clause.clause);
        insta::assert_snapshot!(sql, @"users.id = ?");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], Value::Int(30));
    }
}
