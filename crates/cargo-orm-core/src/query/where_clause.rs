use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{
        query_type::{QueryContext, Value},
        to_sql::ToSql,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum WhereClauseType<'clause> {
    Condition(Condition<'clause>),
    And(Box<WhereClauseType<'clause>>, Box<WhereClauseType<'clause>>),
    Or(Box<WhereClauseType<'clause>>, Box<WhereClauseType<'clause>>),
    Not(Box<WhereClauseType<'clause>>),
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

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause<'clause> {
    pub clause: WhereClauseType<'clause>,
}

impl<'clause> ToSql for WhereClause<'clause> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        ctx.sql.push_str(" WHERE ");
        self.clause.to_sql(ctx, dialect);
    }
}

impl<'clause> WhereClauseType<'clause> {
    /// Renders the clause with optional parentheses based on operator precedence.
    /// Precedence: NOT > AND > OR
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
        match self {
            Condition::Eq(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" = ");
                ctx.push_bind_param(val.clone(), dialect);
            }
            Condition::Ne(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" != ");
                ctx.push_bind_param(val.clone(), dialect);
            }
            Condition::Lt(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" < ");
                ctx.push_bind_param(val.clone(), dialect);
            }
            Condition::Gt(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" > ");
                ctx.push_bind_param(val.clone(), dialect);
            }
            Condition::Lte(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" <= ");
                ctx.push_bind_param(val.clone(), dialect);
            }
            Condition::Gte(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" >= ");
                ctx.push_bind_param(val.clone(), dialect);
            }
            Condition::Like(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" LIKE ");
                ctx.push_bind_param(val.clone(), dialect);
            }
            Condition::NotLike(col, val) => {
                ctx.sql.push_str(col);
                ctx.sql.push_str(" NOT LIKE ");
                ctx.push_bind_param(val.clone(), dialect);
            }
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
