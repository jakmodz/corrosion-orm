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
/// #[derive(Clone, Copy)]
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
    where_clause: Option<WhereClause<C>>,
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
    /// let _sel = Select::new("users");
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
    /// ```no_run
    /// // Add a single column to a Select builder
    /// let sel = Select::new("users").add_column("id");
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
    /// let select = Select::new("users").where_clause(WhereClause::new("id = 1"));
    /// // `select` now contains the provided WHERE clause
    /// ```
    pub fn where_clause(mut self, where_clause: WhereClause<C>) -> Self {
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
    /// let join = /* construct a Join<'_> value */ unimplemented!();
    /// let q = Select::new("users").join(join);
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
    /// # use corrosion_orm_core::query::{Select, OrderBy, OrderDirection};
    /// let select = Select::new("users")
    ///     .add_order_by(OrderBy::new("created_at", OrderDirection::Desc));
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
    pub fn get_where_clause(&self) -> Option<&WhereClause<C>> {
        self.where_clause.as_ref()
    }
    #[cfg(feature = "test-utils")]
    pub fn get_limit(&self) -> Option<usize> {
        self.limit
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
    /// // Pseudocode example — types omitted for brevity
    /// let mut ctx = QueryContext::default();
    /// let sel = Select::new("users").add_column("id");
    /// sel.to_sql(&mut ctx, &SqliteDialect {});
    /// assert_eq!(ctx.sql, "SELECT id FROM users");
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
    /// let schema: TableSchemaModel = /* obtain or construct schema */ unimplemented!();
    /// let sel = Select::<UserColumn>::from(&schema);
    /// assert_eq!(sel.get_table(), schema.name.as_str());
    /// assert!(sel.get_columns().iter().all(|c| c.starts_with(&format!("{}.", schema.name))));
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
            joins: Some(
                schema
                    .relations
                    .iter()
                    .filter(|r| {
                        matches!(
                            r.relation_type,
                            crate::schema::relation::RelationType::HasOne
                                | crate::schema::relation::RelationType::BelongsTo
                        )
                    })
                    .filter_map(|r| Join::from_relation(r).ok())
                    .collect(),
            ),
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
    /// // Given a schema for a `users` table with columns `id` and `name`
    /// let schema = TableSchemaModel::new("users", vec!["id", "name"]);
    /// let select: Select<'_, UserColumn> = Select::from(schema);
    /// assert!(select.columns.contains(&std::borrow::Cow::Owned("users.id".into())));
    /// assert!(select.columns.contains(&std::borrow::Cow::Owned("users.name".into())));
    /// assert_eq!(select.table, std::borrow::Cow::Owned("users".into()));
    /// ```
    fn from(schema: TableSchemaModel) -> Self {
        let mut columns = Vec::new();

        for col in schema.get_column_names() {
            columns.push(Cow::Owned(format!("{}.{}", schema.name, col)));
        }

        Self {
            table: Cow::Owned(schema.name),
            columns,
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
            joins: None,
        }
    }
}
