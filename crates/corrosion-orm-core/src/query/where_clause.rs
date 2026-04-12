use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{
        query_type::{QueryContext, Value},
        to_sql::ToSql,
    },
    types::ColumnTrait,
};

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause<C: ColumnTrait> {
    pub clause: WhereClauseType<C>,
}

impl<C: ColumnTrait> WhereClause<C> {
    pub fn new(clause: WhereClauseType<C>) -> Self {
        Self { clause }
    }

    pub fn eq<V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::eq(col, val)))
    }

    pub fn ne<V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::ne(col, val)))
    }

    pub fn gt<V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::gt(col, val)))
    }

    pub fn lt<V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::lt(col, val)))
    }

    pub fn is_null(col: C) -> Self {
        Self::new(WhereClauseType::Condition(Condition::is_null(col)))
    }

    pub fn in_<V: Into<Value>>(col: C, vals: Vec<V>) -> Self {
        Self::new(WhereClauseType::Condition(Condition::in_(
            col,
            vals.into_iter().map(|v| v.into()).collect(),
        )))
    }

    pub fn and(self, other: WhereClause<C>) -> Self {
        Self::new(WhereClauseType::And(
            Box::new(self.clause),
            Box::new(other.clause),
        ))
    }

    pub fn or(self, other: WhereClause<C>) -> Self {
        Self::new(WhereClauseType::Or(
            Box::new(self.clause),
            Box::new(other.clause),
        ))
    }

    pub fn not_(self) -> Self {
        Self::new(WhereClauseType::Not(Box::new(self.clause)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WhereClauseType<C: ColumnTrait> {
    Condition(Condition<C>),
    And(Box<WhereClauseType<C>>, Box<WhereClauseType<C>>),
    Or(Box<WhereClauseType<C>>, Box<WhereClauseType<C>>),
    Not(Box<WhereClauseType<C>>),
}

impl<C: ColumnTrait> WhereClauseType<C> {
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
                a.render_with_parens(ctx, dialect, matches!(a.as_ref(), WhereClauseType::Or(..)));
                ctx.sql.push_str(" AND ");
                b.render_with_parens(ctx, dialect, matches!(b.as_ref(), WhereClauseType::Or(..)));
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
                    WhereClauseType::Condition(..) | WhereClauseType::Not(..)
                );
                a.render_with_parens(ctx, dialect, child_needs_parens);
            }
        }

        if needs_parens {
            ctx.sql.push(')');
        }
    }
}

impl<C: ColumnTrait> ToSql for WhereClause<C> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        self.clause.to_sql(ctx, dialect);
    }
}

impl<C: ColumnTrait> ToSql for WhereClauseType<C> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        self.render_with_parens(ctx, dialect, false);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition<C: ColumnTrait> {
    Eq(C, Value),
    Ne(C, Value),
    Lt(C, Value),
    Gt(C, Value),
    Lte(C, Value),
    Gte(C, Value),
    IsNull(C),
    In(C, Vec<Value>),
    NotIn(C, Vec<Value>),
    Like(C, Value),
    NotLike(C, Value),
    Between(C, Value, Value),
}

impl<C: ColumnTrait> ToSql for Condition<C> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        macro_rules! binary_op {
            ($col:expr, $val:expr, $op:expr) => {{
                ctx.sql.push_str($col.as_str());
                ctx.sql.push_str($op);
                ctx.push_bind_param($val.clone(), dialect);
            }};
        }

        match self {
            Condition::Eq(c, v) => binary_op!(c, v, " = "),
            Condition::Ne(c, v) => binary_op!(c, v, " != "),
            Condition::Lt(c, v) => binary_op!(c, v, " < "),
            Condition::Gt(c, v) => binary_op!(c, v, " > "),
            Condition::Lte(c, v) => binary_op!(c, v, " <= "),
            Condition::Gte(c, v) => binary_op!(c, v, " >= "),
            Condition::Like(c, v) => binary_op!(c, v, " LIKE "),
            Condition::NotLike(c, v) => binary_op!(c, v, " NOT LIKE "),
            Condition::IsNull(c) => {
                ctx.sql.push_str(c.as_str());
                ctx.sql.push_str(" IS NULL");
            }
            Condition::In(c, vals) => self.render_list_op(ctx, dialect, c.as_str(), " IN ", vals),
            Condition::NotIn(c, vals) => {
                self.render_list_op(ctx, dialect, c.as_str(), " NOT IN ", vals)
            }
            Condition::Between(c, min, max) => {
                ctx.sql.push_str(c.as_str());
                ctx.sql.push_str(" BETWEEN ");
                ctx.push_bind_param(min.clone(), dialect);
                ctx.sql.push_str(" AND ");
                ctx.push_bind_param(max.clone(), dialect);
            }
        }
    }
}
macro_rules! condition_impl {
    ($name:ident, $variant:ident) => {
        pub fn $name<V: Into<Value>>(col: C, value: V) -> Self {
            Condition::$variant(col, value.into())
        }
    };
}
impl<C: ColumnTrait> Condition<C> {
    fn render_list_op(
        &self,
        ctx: &mut QueryContext,
        dialect: &dyn SqlDialect,
        col: &str,
        op: &str,
        vals: &[Value],
    ) {
        ctx.sql.push_str(col);
        ctx.sql.push_str(op);
        ctx.sql.push('(');
        for (i, v) in vals.iter().enumerate() {
            if i > 0 {
                ctx.sql.push_str(", ");
            }
            ctx.push_bind_param(v.clone(), dialect);
        }
        ctx.sql.push(')');
    }

    condition_impl!(eq, Eq);
    condition_impl!(ne, Ne);
    condition_impl!(lt, Lt);
    condition_impl!(gt, Gt);
    condition_impl!(lte, Lte);
    condition_impl!(gte, Gte);
    condition_impl!(like, Like);
    condition_impl!(not_like, NotLike);

    pub fn is_null(col: C) -> Self {
        Condition::IsNull(col)
    }
    pub fn in_(col: C, values: Vec<Value>) -> Self {
        Condition::In(col, values)
    }
    pub fn not_in<V: Into<Value>>(col: C, values: Vec<V>) -> Self {
        Condition::NotIn(col, values.into_iter().map(|v| v.into()).collect())
    }
    pub fn between<V: Into<Value>>(col: C, min: V, max: V) -> Self {
        Condition::Between(col, min.into(), max.into())
    }
}
