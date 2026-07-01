# `#[Column]`

Configures field-to-column mapping and column metadata.

## Targets

- Struct field level

## Syntax

```rust
#[Column(name = "email", unique, nullable, index)]
```

## Parameters

- `name: String` (optional)
  - DB column name. Defaults to field name when omitted.
- `unique: bool` (optional, default `false`)
- `nullable: bool` (optional, default `false`)
- `index: bool` (optional, default `false`)
  - Creates a single-column index.
- `column_definition: String` (optional)
  - Overrides inferred SQL type with a custom SQL type.
- `generation_strategy: GenerationType` (optional)
  - Usually used with primary keys.

## Nullability behavior

A field is treated as nullable if:

- type is `Option<T>`, or
- `nullable = true` is set.

## Example

```rust
#[derive(Model)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,

    #[Column(name = "email", unique, index)]
    pub email: String,

    #[Column(nullable)]
    pub display_name: Option<String>,
}
```
