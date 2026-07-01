# `#[BelongsTo]`

Declares a many-to-one relation (current model belongs to parent model).

## Targets

- Relation field on child struct

## Field type

- Eager: `Parent`
- Lazy: `Lazy<Parent>`

## Syntax

```rust
#[BelongsTo(foreign_key = "user_id", table = "users")]
```

## Parameters

- `foreign_key: Option<String>` (optional)
  - Defaults to `<field_name>_id` if omitted.
- `table: Option<String>` (optional)
- `cascade: bool` (optional, default `true`)

## Important rule

The child model should explicitly declare the FK field as a normal column for type-safe query API usage.

```rust
#[Column(name = "user_id")]
pub user_id: i64,

#[BelongsTo(foreign_key = "user_id", table = "users")]
pub user: User,
```
