# Generated `COLUMN` / `Column` modules

When you derive `Model`, the macro generates a module named after your struct identifier in lowercase.

For `User`, generated module is `user`.

## What gets generated

Inside that module:

1. `pub enum Column`
   - raw column enum implementing `ColumnTrait`
   - variants correspond to model column names

2. `pub struct Columns`
   - typed wrapper fields (`StringColumn`, `NumericColumn`, ...)

3. `pub const COLUMN: Columns`
   - ready-to-use singleton for fluent query API

## Why there are two APIs

### `user::COLUMN` (typed wrappers)

```rust
User::find().filter(user::COLUMN.id.gt(10))
```

- ergonomic
- gives type-specific methods (`contains`, `between`, etc.)

### `user::Column` (raw enum)

```rust
WhereClause::eq(user::Column::id, 1)
```

- lower-level API
- useful when manually constructing `WhereClause`

## Naming conventions and gotchas

- Module name is lowercased struct ident: `UserProfile` -> `userprofile`.
- Wrapper fields in `COLUMN` are lowercased from DB column names.
- Enum variants in `Column` use the configured column names (`#[Column(name = ...)]`) as identifiers.

To avoid awkward names, prefer snake_case column names in `#[Column(name = "...")]`.

## Minimal example

```rust
#[derive(Model)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,
    pub name: String,
}

let by_id = User::find().filter(user::COLUMN.id.eq(1));
let by_name = User::find().filter(WhereClause::eq(user::Column::name, "Alice"));
```
