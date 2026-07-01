# `#[Index]`

Declares a table-level index.

## Targets

- Struct level (can be repeated)

## Syntax

```rust
#[Index(name = "idx_email", fields = ["email"], unique = true)]
```

## Parameters

- `name: String` (optional)
  - Auto-generated if omitted (`idx_<table>_<field1>_<field2>...`).
- `fields: Vec<String>` (required)
- `unique: bool` (optional, default `false`)

## Example

```rust
#[derive(Model)]
#[Table(name = "orders")]
#[Index(fields = ["user_id", "created_at"])]
#[Index(name = "idx_orders_ref", fields = ["external_ref"], unique = true)]
pub struct Order {
    #[PrimaryKey]
    pub id: i64,
    pub user_id: i64,
    pub created_at: String,
    pub external_ref: String,
}
```
