use crate::types::column_ref::ColumnRef;

pub trait ColumnTrait: Copy + Send + Sync {
    /// The name of the table this column belongs to
    fn table_name(&self) -> &'static str;

    /// The bare name of the column
    fn column_name(&self) -> &'static str;

    /// Returns a bare reference: "id"
    fn as_bare(&self) -> ColumnRef {
        ColumnRef::Bare(self.column_name())
    }

    /// Returns a fully qualified reference: "users.id"
    fn as_qualified(&self) -> ColumnRef {
        ColumnRef::Qualified(self.table_name(), self.column_name())
    }
}
