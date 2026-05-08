use std::borrow::Cow;

use crate::{
    dialect::sql_dialect::SqlDialect, prelude::QueryContext, query::ToSql,
    schema::relation::RelationModel,
};

#[derive(Debug, Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone)]
pub struct Join<'query> {
    table: Cow<'query, str>,
    on_left: String,
    on_right: String,
    ty: JoinType,
}

impl<'query> Join<'query> {
    pub fn new(table: Cow<'query, str>, on_left: String, on_right: String, ty: JoinType) -> Self {
        Self {
            table,
            on_left,
            on_right,
            ty,
        }
    }
    pub fn from_relation(r: &'query RelationModel) -> Result<Join<'query>, String> {
        let left = format!("{}.{}", r.source_table, r.foreign_key);
        let right = format!("{}.{}", r.table, r.target_key);
        let ty = match &r.relation_type {
            crate::schema::relation::RelationType::HasOne => crate::query::join::JoinType::Left,
            crate::schema::relation::RelationType::HasMany => crate::query::join::JoinType::Left,
            crate::schema::relation::RelationType::BelongsTo => crate::query::join::JoinType::Inner,
            crate::schema::relation::RelationType::BelongsToMany => {
                return Err("BelongsToMany relations are not currently supported by from_relation as they require a junction table.".to_string());
            }
        };
        Ok(Join::new(Cow::Borrowed(&r.table), left, right, ty))
    }
}

impl<'query> ToSql for Join<'query> {
    fn to_sql(&self, ctx: &mut QueryContext, _dialect: &dyn SqlDialect) {
        match self.ty {
            JoinType::Inner => {}
            JoinType::Left => ctx.sql.push_str(" LEFT"),
            JoinType::Right => ctx.sql.push_str(" RIGHT"),
            JoinType::Full => ctx.sql.push_str(" FULL"),
        }
        ctx.sql.push_str(" JOIN ");
        ctx.sql.push_str(&self.table);
        ctx.sql.push_str(" ON ");
        ctx.sql.push_str(&self.on_left);
        ctx.sql.push_str(" = ");
        ctx.sql.push_str(&self.on_right);
    }
}
