# `#[HasOne]`

Declares a one-to-one relation.

## Targets

- Relation field on owner struct

## Field type

- Eager: `Related`
- Lazy: `Lazy<Related>`

## Syntax

```rust
#[HasOne]
```

or

```rust
#[HasOne(foreign_key = "profile_id", table = "profiles", cascade = true)]
```

## Parameters

- `foreign_key: Option<String>` (optional)
  - Defaults to `<field_name>_id` if omitted.
- `table: Option<String>` (optional)
  - Related table name override.
- `cascade: bool` (optional, default `true`)

## Notes

- Treated as unique relation in generated schema metadata.
- For typed filtering convenience, keep explicit FK columns where appropriate.
