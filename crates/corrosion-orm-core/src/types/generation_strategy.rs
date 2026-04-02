#[derive(deluxe::ParseMetaItem, Debug, PartialEq, Eq)]
/// Represents the generation strategy for a column.
pub enum GenerationType {
    /// Automatically generates a unique identifier for each row.
    AutoIncrement,
}
