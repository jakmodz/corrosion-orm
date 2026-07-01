# `#[Table]`

Marks a struct as a database table and optionally overrides the SQL table name.

## Targets

- Struct level

## Syntax

```rust
#[Table(name = "users")]
```

## Parameters

- `name: String` (optional)
  - If omitted, defaults to the Rust struct name.

## Example

```rust
#[derive(Model)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,
}
```
