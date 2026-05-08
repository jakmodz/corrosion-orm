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
    /// Constructs a `Join` representing a SQL join against `table` with the given ON expressions and join type.
    ///
    /// `table` may be borrowed or owned (`Cow<'query, str>`). `on_left` and `on_right` are the left and right
    /// expressions used in the join equality (e.g., `"users.id"` and `"posts.user_id"`). `ty` selects the join kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use corrosion_orm_core::query::join::{Join, JoinType};
    /// let j = Join::new(Cow::Borrowed("posts"), "users.id".to_string(), "posts.user_id".to_string(), JoinType::Left);
    /// assert_eq!(j.table(), "posts");
    /// assert_eq!(j.on_left(), "users.id");
    /// assert_eq!(j.on_right(), "posts.user_id");
    /// matches!(j.ty(), JoinType::Left);
    /// ```
    pub fn new(table: Cow<'query, str>, on_left: String, on_right: String, ty: JoinType) -> Self {
        Self {
            table,
            on_left,
            on_right,
            ty,
        }
    }

    pub fn table(&self) -> &str {
        &self.table
    }

    pub fn on_left(&self) -> &str {
        &self.on_left
    }

    pub fn on_right(&self) -> &str {
        &self.on_right
    }

    pub fn ty(&self) -> &JoinType {
        &self.ty
    }
    /// Create a `Join` derived from a relationship definition.
    ///
    /// Constructs a `Join` whose ON expressions and join kind are determined by the
    /// provided `RelationModel`. Returns `Ok(Join)` for supported relation types:
    /// - `HasOne` and `HasMany` produce a `LEFT` join.
    /// - `BelongsTo` produces an `INNER` join.
    ///
    /// Returns `Err` if `BelongsToMany` is encountered because many-to-many relations
    /// require a junction table and are not supported by this constructor.
    ///
    /// # Returns
    ///
    /// `Ok(Join)` with the joined table and ON expressions on success, `Err(String)`
    /// with an explanatory message for unsupported relation types.
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
    /// Appends this join's SQL fragment to the given query context.
    ///
    /// This writes an optional join modifier (`LEFT`, `RIGHT`, `FULL`) followed by
    /// `" JOIN <table> ON <left> = <right>"` into `ctx.sql`. The `_dialect`
    /// parameter is ignored by this implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use corrosion_orm_core::prelude::*;
    /// use corrosion_orm_core::query::join::{Join, JoinType};
    ///
    /// // Construct a join representing: LEFT JOIN users ON posts.user_id = users.id
    /// let join = Join::new(
    ///     Cow::Borrowed("users"),
    ///     "posts.user_id".to_string(),
    ///     "users.id".to_string(),
    ///     JoinType::Left,
    /// );
    ///
    /// let mut ctx = QueryContext::default();
    /// # #[cfg(feature = "sqlite")]
    /// # {
    /// # use corrosion_orm_core::dialect::sqlite_dialect::SqliteDialect;
    /// join.to_sql(&mut ctx, &SqliteDialect);
    /// assert!(ctx.sql.contains("LEFT JOIN users ON posts.user_id = users.id"));
    /// # }
    /// ```
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
