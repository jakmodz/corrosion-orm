use std::borrow::Cow;

use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{
        join::Join,
        order_by::{OrderBy, OrderClause},
        to_sql::ToSql,
    },
    schema::table::TableSchemaModel,
    types::ColumnTrait,
};

use super::where_clause::WhereClause;

#[derive(Debug, Clone)]
/// SELECT query builder.
///
/// Builds SELECT queries with optional WHERE clauses and LIMIT.
///
/// The `C` generic type must implement `ColumnTrait` to guarantee type-safe
/// column references in the `WhereClause`. This replaces raw strings with
/// compile-time validated enum variants.
///
/// # Examples
///
/// ```
/// use corrosion_orm_core::query::select::Select;
/// use corrosion_orm_core::query::where_clause::WhereClause;
/// use corrosion_orm_core::types::ColumnTrait;
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// enum UserColumn { Id, Name }
/// impl ColumnTrait for UserColumn {
///     fn table_name(&self) -> &'static str { "users" }
///     fn column_name(&self) -> &'static str {
///         match self { Self::Id => "id", Self::Name => "name" }
///     }
/// }
///
/// let query = Select::<UserColumn>::new("users")
///     .add_column("id")
///     .add_column("name")
///     .where_clause(WhereClause::eq(UserColumn::Id, 1));
/// ```
pub struct Select<'query, C: ColumnTrait> {
    table: Cow<'query, str>,
    columns: Vec<Cow<'query, str>>,
    where_clause: Option<WhereClause>,
    order_by: Option<OrderClause<C>>,
    limit: Option<usize>,
    offset: Option<usize>,
    joins: Option<Vec<Join<'query>>>,
}

impl<'col, C: ColumnTrait> Select<'col, C> {
    /// Creates a new SELECT query builder for the specified table.
    ///
    /// The provided `table` is converted into a `Cow<'col, str>` and stored; the returned builder
    /// starts with no selected columns, no WHERE or ORDER BY clauses, and no limit, offset, or joins.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::query::select::Select;
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy)] enum MockColumn {}
    /// # impl ColumnTrait for MockColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "id" }
    /// # }
    /// let _sel = Select::<MockColumn>::new("users");
    /// ```
    pub fn new<T: Into<Cow<'col, str>>>(table: T) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
            joins: None,
        }
    }
    /// Appends a column expression to the SELECT projection and returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::query::select::Select;
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy)] enum MockColumn {}
    /// # impl ColumnTrait for MockColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "id" }
    /// # }
    /// // Add a single column to a Select builder
    /// let sel = Select::<MockColumn>::new("users").add_column("id");
    /// ```
    pub fn add_column<Column: Into<Cow<'col, str>>>(mut self, column: Column) -> Self {
        self.columns.push(column.into());
        self
    }
    pub fn columns<S: Into<Cow<'col, str>>>(mut self, columns: Vec<S>) -> Self {
        self.columns = columns.into_iter().map(|c| c.into()).collect();
        self
    }
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    /// Sets the typed WHERE clause for the query builder.
    ///
    /// This attaches the given `WhereClause<C>` to the `Select` and returns the updated builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::query::select::Select;
    /// use corrosion_orm_core::query::where_clause::WhereClause;
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum MockColumn { Id }
    /// # impl ColumnTrait for MockColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "id" }
    /// # }
    /// let select: Select<'_, MockColumn> = Select::new("users").where_clause(WhereClause::eq(MockColumn::Id, 1));
    /// ```
    pub fn where_clause(mut self, where_clause: WhereClause) -> Self {
        self.where_clause = Some(where_clause);
        self
    }
    /// Appends the provided `Join` to the builder's joins list and returns the updated `Select`.
    ///
    /// If the builder already has joins, the new join is appended; otherwise the joins list is created.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use corrosion_orm_core::query::select::Select;
    /// use corrosion_orm_core::query::join::{Join, JoinType};
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy)] enum MockColumn {}
    /// # impl ColumnTrait for MockColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "id" }
    /// # }
    ///
    /// let join = Join::new(Cow::Borrowed("posts"), "users.id".to_string(), "posts.user_id".to_string(), JoinType::Left);
    /// let q = Select::<MockColumn>::new("users").join(join);
    /// ```
    pub fn join(mut self, join: Join<'col>) -> Self {
        if let Some(joins) = &mut self.joins {
            joins.push(join);
        } else {
            self.joins = Some(vec![join]);
        }
        self
    }
    /// Adds an `OrderBy` expression to the SELECT builder's ORDER BY clause.
    ///
    /// If the builder already has an ORDER BY clause, the provided `order_by` is appended to it;
    /// otherwise a new ORDER BY clause is created containing `order_by`.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::query::select::Select;
    /// use corrosion_orm_core::query::order_by::{OrderBy, OrderDirection};
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum MockColumn { CreatedAt }
    /// # impl ColumnTrait for MockColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "created_at" }
    /// # }
    /// let select = Select::new("users")
    ///     .add_order_by(OrderBy::new(MockColumn::CreatedAt, OrderDirection::Desc));
    /// ```
    pub fn add_order_by(mut self, order_by: OrderBy<C>) -> Self {
        if let Some(order_clause) = &mut self.order_by {
            order_clause.columns.push(order_by);
        } else {
            self.order_by = Some(OrderClause {
                columns: vec![order_by],
            });
        }
        self
    }

    #[cfg(feature = "test-utils")]
    pub fn get_table(&self) -> &str {
        &self.table
    }
    #[cfg(feature = "test-utils")]
    pub fn get_columns(&self) -> &[Cow<'col, str>] {
        &self.columns
    }
    #[cfg(feature = "test-utils")]
    pub fn get_where_clause(&self) -> Option<&WhereClause> {
        self.where_clause.as_ref()
    }
    #[cfg(feature = "test-utils")]
    pub fn get_limit(&self) -> Option<usize> {
        self.limit
    }

    /// Build JOIN clauses for all eager HasOne / BelongsTo relations in the schema.
    fn build_eager_joins(schema: &'col TableSchemaModel) -> Option<Vec<Join<'col>>> {
        let joins: Vec<Join> = schema
            .relations
            .iter()
            .filter(|r| {
                r.is_eager
                    && matches!(
                        r.relation_type,
                        crate::schema::relation::RelationType::HasOne
                            | crate::schema::relation::RelationType::BelongsTo
                    )
            })
            .filter_map(|r| Join::from_relation(r).ok())
            .collect();
        if joins.is_empty() { None } else { Some(joins) }
    }
}
impl<C: ColumnTrait> ToSql for Select<'_, C> {
    /// Appends this SELECT builder's SQL representation to the provided query context.
    ///
    /// The generated SQL includes the selected columns (or `*` when none are specified),
    /// the FROM table, any configured joins (rendered after the FROM clause), an optional
    /// WHERE clause, optional ORDER BY clause, and optional LIMIT and OFFSET clauses,
    /// in that order.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::prelude::*;
    /// use corrosion_orm_core::query::select::Select;
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy)] enum MockColumn {}
    /// # impl ColumnTrait for MockColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "id" }
    /// # }
    ///
    /// let mut ctx = QueryContext::default();
    /// let sel = Select::<MockColumn>::new("users").add_column("id");
    /// # #[cfg(feature = "sqlite")]
    /// # {
    /// # use corrosion_orm_core::dialect::sqlite_dialect::SqliteDialect;
    /// sel.to_sql(&mut ctx, &SqliteDialect);
    /// assert_eq!(ctx.sql, "SELECT id FROM users");
    /// # }
    /// ```
    fn to_sql(&self, ctx: &mut super::query_type::QueryContext, _dialect: &dyn SqlDialect) {
        ctx.sql.push_str(&format!(
            "SELECT {} FROM {}",
            if self.columns.is_empty() {
                String::from("*")
            } else {
                self.columns.join(", ")
            },
            self.table
        ));
        if let Some(joins) = &self.joins {
            for join in joins {
                join.to_sql(ctx, _dialect);
            }
        }
        if let Some(where_clause) = &self.where_clause {
            ctx.sql.push_str(" WHERE ");
            where_clause.to_sql(ctx, _dialect);
        }
        if let Some(order_by) = &self.order_by {
            ctx.sql.push_str(" ORDER BY ");
            order_by.to_sql(ctx, _dialect);
        }
        if let Some(limit) = self.limit {
            ctx.sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            ctx.sql.push_str(&format!(" OFFSET {}", offset));
        }
    }
}
impl<'col, C: ColumnTrait> From<&'col TableSchemaModel> for Select<'col, C> {
    /// Builds a `Select` configured from a borrowed `TableSchemaModel`.
    ///
    /// The resulting builder uses a borrowed table name, creates owned column expressions
    /// in the form `"table.column"` for every column in the schema, leaves `where_clause`,
    /// `order_by`, `limit`, and `offset` unset, and populates `joins` with join descriptions
    /// derived from the schema's relations that are `HasOne` or `BelongsTo`.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::prelude::*;
    /// use corrosion_orm_core::query::select::Select;
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum UserColumn { Id }
    /// # impl ColumnTrait for UserColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "id" }
    /// # }
    ///
    /// let schema = TableSchemaModel::new("users".to_string());
    /// let sel = Select::<UserColumn>::from(&schema);
    /// # #[cfg(feature = "test-utils")]
    /// # {
    /// assert_eq!(sel.get_table(), schema.name.as_str());
    /// # }
    /// ```
    fn from(schema: &'col TableSchemaModel) -> Self {
        Self {
            table: Cow::Borrowed(&schema.name),
            columns: schema
                .get_column_names()
                .into_iter()
                .map(|col| Cow::Owned(format!("{}.{}", schema.name, col)))
                .collect(),
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
            joins: Self::build_eager_joins(schema),
        }
    }
}

impl<'col, C: ColumnTrait> From<TableSchemaModel> for Select<'col, C> {
    /// Builds a `Select` that owns the schema's table name and all its columns.
    ///
    /// The resulting `Select` has:
    /// - `table` set to the schema's name (owned),
    /// - `columns` containing owned `"table.column"` strings for every column returned by `schema.get_column_names()`,
    /// - `where_clause`, `order_by`, `limit`, `offset`, and `joins` set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::prelude::*;
    /// use corrosion_orm_core::query::select::Select;
    /// # use corrosion_orm_core::types::ColumnTrait;
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum UserColumn { Id }
    /// # impl ColumnTrait for UserColumn {
    /// #     fn table_name(&self) -> &'static str { "users" }
    /// #     fn column_name(&self) -> &'static str { "id" }
    /// # }
    ///
    /// let schema = TableSchemaModel::new("users".to_string());
    /// let select: Select<'_, UserColumn> = Select::from(schema);
    /// # #[cfg(feature = "test-utils")]
    /// # {
    /// assert_eq!(select.get_table(), "users");
    /// # }
    /// ```
    fn from(schema: TableSchemaModel) -> Self {
        let mut columns = Vec::new();

        for col in schema.get_column_names() {
            columns.push(Cow::Owned(format!("{}.{}", schema.name, col)));
        }

        let joins: Option<Vec<Join>> = {
            let j: Vec<Join> = schema
                .relations
                .iter()
                .filter(|r| {
                    r.is_eager
                        && matches!(
                            r.relation_type,
                            crate::schema::relation::RelationType::HasOne
                                | crate::schema::relation::RelationType::BelongsTo
                        )
                })
                .map(|r| {
                    let left = format!("{}.{}", r.source_table, r.foreign_key);
                    let right = format!("{}.{}", r.table, r.target_key);
                    let ty = match r.relation_type {
                        crate::schema::relation::RelationType::HasOne
                        | crate::schema::relation::RelationType::HasMany => {
                            crate::query::join::JoinType::Left
                        }
                        crate::schema::relation::RelationType::BelongsTo => {
                            crate::query::join::JoinType::Inner
                        }
                        _ => crate::query::join::JoinType::Left,
                    };
                    Join::new(Cow::Owned(r.table.clone()), left, right, ty)
                })
                .collect();
            if j.is_empty() { None } else { Some(j) }
        };

        Self {
            table: Cow::Owned(schema.name),
            columns,
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
            joins,
        }
    }
}
