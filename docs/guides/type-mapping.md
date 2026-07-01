# Type Mapping

This page documents how Rust field types are mapped to Corrosion ORM internal SQL types and then to SQLite column types.

## Mapping table

| Rust field type | Internal `SqlType` | SQLite DDL type | Generated wrapper in `<model>::COLUMN` | Notes |
|---|---|---|---|---|
| `String` | `Varchar(255)` | `TEXT` | `StringColumn` | Default mapping for `String` fields. |
| `Option<String>` | `Varchar(255)` | `TEXT` | `StringColumn` | Nullable because it is `Option<_>`. |
| `bool` | `Boolean` | `INTEGER` | `BooleanColumn` | SQLite stores booleans as integers. |
| `Option<bool>` | `Boolean` | `INTEGER` | `BooleanColumn` | Nullable boolean. |
| `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64` | `Integer` | `INTEGER` | `NumericColumn` | Standard integer mapping. |
| `f32` | `Float` | `REAL` | `NumericColumn` | |
| `f64` | `Double` | `REAL` | `NumericColumn` | |
| `chrono::NaiveDate` | `Date` | `DATE` | `DateLikeColumn` | |
| `chrono::NaiveDateTime` | `Timestamp` | `TIMESTAMP` | `DateLikeColumn` | |
| `Option<T>` | same as `T` | same as `T` | same as `T` | `Option` affects nullability, not SQL base type. |
| `#[Column(column_definition = "...")]` | `Custom("...")` | custom | wrapper chosen heuristically | Use for manual SQL type override. |

## Important behavior details

1. **Nullability rules**
   - A field is nullable if either:
     - it is `Option<T>`, or
     - `#[Column(nullable)]` is set.

2. **String in SQLite**
   - `Char(_)`, `Varchar(_)`, and `Text` all become `TEXT` in SQLite dialect.

3. **Custom SQL types**
   - If you set `#[Column(column_definition = "NUMERIC(10,2)")]`, schema uses that exact SQL type.
   - Generated `COLUMN` wrappers for custom types are inferred heuristically and may not always match intended numeric/date behavior.

4. **Default-based schema generation**
   - Generated schema uses `<FieldType>::default()` internally for type-to-SQL conversion.
   - In practice, model field types should implement `Default`.

## Example

```rust
#[derive(Model)]
#[Table(name = "products")]
pub struct Product {
    #[PrimaryKey]
    pub id: i64,

    pub name: String,            // Varchar(255) -> TEXT (sqlite)
    pub active: bool,            // Boolean -> INTEGER
    pub weight: f32,             // Float -> REAL

    #[Column(column_definition = "NUMERIC(10,2)")]
    pub price: String,           // Custom override
}
```

## Recommendation

If your database portability requirements grow, keep custom DB-specific types explicit with `column_definition` and document those fields next to the model.
