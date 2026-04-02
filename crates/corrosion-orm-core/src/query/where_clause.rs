use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{
        query_type::{QueryContext, Value},
        to_sql::ToSql,
    },
};
/// A WHERE clause builder for composing SQL conditions.
///
/// # Examples
/// ```ignore
/// WhereClause::eq("age", 18)
///     .and(WhereClause::eq("status", "active"))
///     .or(WhereClause::gt("status", "inactive"))
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause<'clause> {
    pub clause: WhereClauseType<'clause>,
}
impl<'clause> WhereClause<'clause> {
    pub fn new(clause: WhereClauseType<'clause>) -> Self {
        Self { clause }
    }
    pub fn eq<V: Into<Value>>(col: &'clause str, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::eq(col, val)))
    }

    pub fn ne<V: Into<Value>>(col: &'clause str, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::ne(col, val)))
    }

    pub fn gt<V: Into<Value>>(col: &'clause str, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::gt(col, val)))
    }

    pub fn lt<V: Into<Value>>(col: &'clause str, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::lt(col, val)))
    }

    pub fn is_null(col: &'clause str) -> Self {
        Self::new(WhereClauseType::Condition(Condition::is_null(col)))
    }

    pub fn in_(col: &'clause str, vals: Vec<Value>) -> Self {
        Self::new(WhereClauseType::Condition(Condition::in_(col, vals)))
    }

    pub fn and(self, other: WhereClause<'clause>) -> Self {
        Self {
            clause: WhereClauseType::And(Box::new(self.clause), Box::new(other.clause)),
        }
    }

    pub fn or(self, other: WhereClause<'clause>) -> Self {
        Self {
            clause: WhereClauseType::Or(Box::new(self.clause), Box::new(other.clause)),
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        Self {
            clause: WhereClauseType::Not(Box::new(self.clause)),
        }
    }
}

impl<'clause> ToSql for WhereClause<'clause> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        self.clause.to_sql(ctx, dialect);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WhereClauseType<'clause> {
    Condition(Condition<'clause>),
    And(Box<WhereClauseType<'clause>>, Box<WhereClauseType<'clause>>),
    Or(Box<WhereClauseType<'clause>>, Box<WhereClauseType<'clause>>),
    Not(Box<WhereClauseType<'clause>>),
}
impl<'clause> From<WhereClauseType<'clause>> for WhereClause<'clause> {
    fn from(clause: WhereClauseType<'clause>) -> Self {
        Self { clause }
    }
}
impl<'clause> WhereClauseType<'clause> {
    fn render_with_parens(
        &self,
        ctx: &mut QueryContext,
        dialect: &dyn SqlDialect,
        needs_parens: bool,
    ) {
        if needs_parens {
            ctx.sql.push('(');
        }

        match self {
            WhereClauseType::Condition(c) => c.to_sql(ctx, dialect),
            WhereClauseType::And(a, b) => {
                let left_needs_parens = matches!(a.as_ref(), WhereClauseType::Or(_, _));
                a.render_with_parens(ctx, dialect, left_needs_parens);
                ctx.sql.push_str(" AND ");
                let right_needs_parens = matches!(b.as_ref(), WhereClauseType::Or(_, _));
                b.render_with_parens(ctx, dialect, right_needs_parens);
            }
            WhereClauseType::Or(a, b) => {
                a.render_with_parens(ctx, dialect, false);
                ctx.sql.push_str(" OR ");
                b.render_with_parens(ctx, dialect, false);
            }
            WhereClauseType::Not(a) => {
                ctx.sql.push_str("NOT ");
                let child_needs_parens = !matches!(
                    a.as_ref(),
                    WhereClauseType::Condition(_) | WhereClauseType::Not(_)
                );
                a.render_with_parens(ctx, dialect, child_needs_parens);
            }
        }

        if needs_parens {
            ctx.sql.push(')');
        }
    }
}

impl<'clause> ToSql for WhereClauseType<'clause> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        self.render_with_parens(ctx, dialect, false);
    }
}

impl<'clause> ToSql for Condition<'clause> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        macro_rules! binary_op {
            ($col:expr, $val:expr, $op:expr) => {{
                ctx.sql.push_str($col);
                ctx.sql.push_str($op);
                ctx.push_bind_param($val.clone(), dialect);
            }};
        }

        match self {
            Condition::Eq(col, val) => binary_op!(col, val, " = "),
            Condition::Ne(col, val) => binary_op!(col, val, " != "),
            Condition::Lt(col, val) => binary_op!(col, val, " < "),
            Condition::Gt(col, val) => binary_op!(col, val, " > "),
            Condition::Lte(col, val) => binary_op!(col, val, " <= "),
            Condition::Gte(col, val) => binary_op!(col, val, " >= "),
            Condition::Like(col, val) => binary_op!(col, val, " LIKE "),
            Condition::NotLike(col, val) => binary_op!(col, val, " NOT LIKE "),
            Condition::In(col, values) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" IN (");
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        ctx.sql.push_str(", ");
                    }
                    ctx.push_bind_param(value.clone(), dialect);
                }
                ctx.sql.push(')');
            }
            Condition::NotIn(col, values) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" NOT IN (");
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        ctx.sql.push_str(", ");
                    }
                    ctx.push_bind_param(value.clone(), dialect);
                }
                ctx.sql.push(')');
            }
            Condition::IsNull(col) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" IS NULL");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition<'clause> {
    Eq(&'clause str, Value),
    Ne(&'clause str, Value),
    Lt(&'clause str, Value),
    Gt(&'clause str, Value),
    Lte(&'clause str, Value),
    Gte(&'clause str, Value),
    IsNull(&'clause str),
    In(&'clause str, Vec<Value>),
    NotIn(&'clause str, Vec<Value>),
    Like(&'clause str, Value),
    NotLike(&'clause str, Value),
}
macro_rules! condition_impl {
    ($name:ident, $variant:ident) => {
        pub fn $name<V: Into<Value>>(col: &'clause str, value: V) -> Self {
            Condition::$variant(col, value.into())
        }
    };
}
impl<'clause> Condition<'clause> {
    condition_impl!(eq, Eq);
    condition_impl!(ne, Ne);
    condition_impl!(lt, Lt);
    condition_impl!(gt, Gt);
    condition_impl!(lte, Lte);
    condition_impl!(gte, Gte);
    condition_impl!(like, Like);
    condition_impl!(not_like, NotLike);
    pub fn is_null(col: &'clause str) -> Self {
        Condition::IsNull(col)
    }
    pub fn in_(col: &'clause str, values: Vec<Value>) -> Self {
        Condition::In(col, values)
    }
    pub fn not_in(col: &'clause str, values: Vec<Value>) -> Self {
        Condition::NotIn(col, values)
    }
}
