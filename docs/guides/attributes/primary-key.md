# `#[PrimaryKey]`

Marks exactly one field as the table primary key.

## Targets

- Struct field level

## Syntax

```rust
#[PrimaryKey]
```

Or with generation strategy:

```rust
#[PrimaryKey(generation_strategy = {auto_increment})]
```

## Parameters

- `generation_strategy: GenerationType` (optional)

## Rules

- A model must have exactly one `#[PrimaryKey]` field.
- Multiple primary keys are rejected by macro parsing.

## Example

```rust
#[derive(Model)]
#[Table(name = "users")]
pub struct User {
    #[Column(name = "id")]
    #[PrimaryKey(generation_strategy = {auto_increment})]
    pub id: i64,
}
```
