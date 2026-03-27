use crate::{
    dialect::sql_dialect::SqlDialect,
    prelude::TableSchemaModel,
    query::{
        query_type::{QueryContext, Value},
        to_sql::ToSql,
        where_clause::WhereClause,
    },
};
use std::borrow::Cow;

pub struct Update<'query> {
    table: Cow<'query, str>,
    columns: Vec<Cow<'query, str>>,
    values: Vec<Value>,
    where_clause: Option<WhereClause<'query>>,
}
impl<'query> Update<'query> {
    pub fn from_model(model: &'query TableSchemaModel) -> Self {
        Self {
            table: Cow::Owned(model.name.clone()),
            columns: model
                .fields
                .iter()
                .map(|f| Cow::Owned(f.name.clone()))
                .collect(),
            values: Vec::new(),
            where_clause: None,
        }
    }
}
impl<'query> ToSql for Update<'query> {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        ctx.sql
            .push_str(&format!("UPDATE {} SET ", self.table.as_ref()));
        let mut iter = self.columns.iter().zip(self.values.iter());
        while let Some((column, value)) = iter.next() {
            ctx.sql.push_str(&format!("{} = ", column.as_ref()));
            ctx.push_bind_param(value.clone(), dialect);
            if iter.next().is_some() {
                ctx.sql.push_str(", ");
            }
        }

        if let Some(where_clause) = &self.where_clause {
            where_clause.to_sql(ctx, dialect);
        }
    }
}
impl<'query> From<TableSchemaModel> for Update<'query> {
    fn from(value: TableSchemaModel) -> Self {
        Update {
            table: Cow::Owned(value.name),
            columns: value
                .fields
                .into_iter()
                .map(|f| Cow::Owned(f.name))
                .collect(),
            values: vec![],
            where_clause: None,
        }
    }
}
