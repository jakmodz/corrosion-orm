use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{
        query_type::{QueryContext, Value},
        to_sql::ToSql,
    },
    types::{ColumnTrait, column_ref::ColumnRef},
};

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub clause: WhereClauseType,
}

impl WhereClause {
    pub fn new(clause: WhereClauseType) -> Self {
        Self { clause }
    }

    pub fn eq<C: ColumnTrait, V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::eq(col, val)))
    }

    pub fn ne<C: ColumnTrait, V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::ne(col, val)))
    }

    pub fn gt<C: ColumnTrait, V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::gt(col, val)))
    }

    pub fn lt<C: ColumnTrait, V: Into<Value>>(col: C, val: V) -> Self {
        Self::new(WhereClauseType::Condition(Condition::lt(col, val)))
    }

    pub fn is_null<C: ColumnTrait>(col: C) -> Self {
        Self::new(WhereClauseType::Condition(Condition::is_null(col)))
    }

    pub fn in_<C: ColumnTrait, V: Into<Value>>(col: C, vals: Vec<V>) -> Self {
        Self::new(WhereClauseType::Condition(Condition::in_(
            col,
            vals.into_iter().map(|v| v.into()).collect(),
        )))
    }

    pub fn and(self, other: WhereClause) -> Self {
        Self::new(WhereClauseType::And(
            Box::new(self.clause),
            Box::new(other.clause),
        ))
    }

    pub fn or(self, other: WhereClause) -> Self {
        Self::new(WhereClauseType::Or(
            Box::new(self.clause),
            Box::new(other.clause),
        ))
    }

    pub fn not_(self) -> Self {
        Self::new(WhereClauseType::Not(Box::new(self.clause)))
    }

    pub fn seek_after<C: ColumnTrait>(
        sort_col: C,
        last_val: impl Into<Value>,
        unique_col: C,
        last_unique_val: impl Into<Value>,
    ) -> Self {
        let last_val = last_val.into();
        let last_unique_val = last_unique_val.into();

        let greater = Self::gt(sort_col, last_val.clone());
        let tie_breaker = Self::eq(sort_col, last_val).and(Self::gt(unique_col, last_unique_val));

        greater.or(tie_breaker)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WhereClauseType {
    Condition(Condition),
    And(Box<WhereClauseType>, Box<WhereClauseType>),
    Or(Box<WhereClauseType>, Box<WhereClauseType>),
    Not(Box<WhereClauseType>),
}

impl WhereClauseType {
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

impl ToSql for WhereClause {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        self.clause.to_sql(ctx, dialect);
    }
}

impl ToSql for WhereClauseType {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        self.render_with_parens(ctx, dialect, false);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Eq(ColumnRef, Value),
    Ne(ColumnRef, Value),
    Lt(ColumnRef, Value),
    Gt(ColumnRef, Value),
    Lte(ColumnRef, Value),
    Gte(ColumnRef, Value),
    IsNull(ColumnRef),
    In(ColumnRef, Vec<Value>),
    NotIn(ColumnRef, Vec<Value>),
    Like(ColumnRef, Value),
    NotLike(ColumnRef, Value),
    Between(ColumnRef, Value, Value),
}

impl ToSql for Condition {
    /// Render this condition as a SQL expression into the given query context.
    ///
    /// The method writes SQL text into `ctx.sql` and pushes any parameter values
    /// into the context as bind parameters using the provided SQL dialect. Column
    /// references are rendered as qualified identifiers when applicable.
    ///
    /// # Parameters
    ///
    /// - `ctx`: Target query context receiving SQL text and bind parameters.
    /// - `dialect`: SQL dialect used when formatting bind parameters.
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        macro_rules! binary_op {
            ($col:expr, $val:expr, $op:expr) => {{
                $col.render(ctx);
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
                c.render(ctx);
                ctx.sql.push_str(" IS NULL");
            }
            Condition::In(c, vals) => self.render_list_op(ctx, dialect, c, " IN ", vals),
            Condition::NotIn(c, vals) => self.render_list_op(ctx, dialect, c, " NOT IN ", vals),
            Condition::Between(c, min, max) => {
                c.render(ctx);
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
        pub fn $name<C: ColumnTrait, V: Into<Value>>(col: C, value: V) -> Self {
            Condition::$variant(col.as_qualified(), value.into())
        }
    };
}
impl Condition {
    /// Render a column membership predicate ("IN"/"NOT IN") by emitting the qualified column,
    /// the operator, and a parenthesized, comma-separated list of bound values.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Example of internal usage:
    /// // cond.render_list_op(&mut ctx, dialect, &col, " IN ", &vals);
    /// ```
    fn render_list_op(
        &self,
        ctx: &mut QueryContext,
        dialect: &dyn SqlDialect,
        col: &ColumnRef,
        op: &str,
        vals: &[Value],
    ) {
        col.render(ctx);
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

    pub fn is_null<C: ColumnTrait>(col: C) -> Self {
        Condition::IsNull(col.as_qualified())
    }
    pub fn in_<C: ColumnTrait>(col: C, values: Vec<Value>) -> Self {
        Condition::In(col.as_qualified(), values)
    }
    pub fn not_in<C: ColumnTrait, V: Into<Value>>(col: C, values: Vec<V>) -> Self {
        Condition::NotIn(
            col.as_qualified(),
            values.into_iter().map(|v| v.into()).collect(),
        )
    }
    pub fn between<C: ColumnTrait, V: Into<Value>>(col: C, min: V, max: V) -> Self {
        Condition::Between(col.as_qualified(), min.into(), max.into())
    }
}
