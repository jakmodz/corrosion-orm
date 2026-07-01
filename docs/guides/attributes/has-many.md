# `#[HasMany]`

Declares a one-to-many relation from parent to children.

## Targets

- Relation field on parent struct

## Field type

- Eager: `Vec<Child>`
- Lazy: `LazyCollection<Child, child::Column>`

## Syntax

```rust
#[HasMany(foreign_key = "user_id", table = "posts")]
```

## Parameters

- `foreign_key: Option<String>` (optional)
  - Defaults to `<field_name>_id` if omitted.
- `table: Option<String>` (optional)
- `cascade: bool` (optional, default `true`)

## Notes

- `HasMany` is virtual from parent table perspective (FK typically lives on child table).
- Lazy collection requires explicit `.load(&mut conn).await?`.
