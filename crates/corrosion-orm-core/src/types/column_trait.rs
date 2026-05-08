use crate::types::column_ref::ColumnRef;

pub trait ColumnTrait: Copy + Send + Sync {
    /// The name of the table this column belongs to
    fn table_name(&self) -> &'static str;

    /// The bare name of the column
    fn column_name(&self) -> &'static str;

    /// Constructs a bare (unqualified) column reference for this column (e.g., `"id"`).
    ///
    /// # Examples
    ///
    /// ```
    /// // assuming a type `MyColumn` implements `ColumnTrait`
    /// let r = MyColumn::Id.as_bare();
    /// assert_eq!(r, ColumnRef::Bare("id"));
    /// ```
    fn as_bare(&self) -> ColumnRef {
        ColumnRef::Bare(self.column_name())
    }

    /// Constructs a qualified column reference combining the column's table and name.
    ///
    /// Returns a `ColumnRef::Qualified` containing the table name and the column name.
    ///
    /// # Examples
    ///
    /// ```
    /// struct UserId;
    ///
    /// impl ColumnTrait for UserId {
    ///     fn table_name(&self) -> &'static str { "users" }
    ///     fn column_name(&self) -> &'static str { "id" }
    /// }
    ///
    /// let col = UserId;
    /// let qualified = col.as_qualified();
    /// assert_eq!(qualified, ColumnRef::Qualified("users", "id"));
    /// ```
    fn as_qualified(&self) -> ColumnRef {
        ColumnRef::Qualified(self.table_name(), self.column_name())
    }
}
