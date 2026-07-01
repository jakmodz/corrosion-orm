# Filtering

Filtering is built on `Finder::filter(...)` and `WhereClause`.

## Two styles

### 1) Column wrapper style (most common)

```rust
let users = User::find()
    .filter(user::COLUMN.id.gt(50))
    .all(&mut conn)
    .await?;
```

### 2) Raw `WhereClause` + enum columns

```rust
let users = User::find()
    .filter(
        WhereClause::eq(user::Column::id, 1)
            .and(WhereClause::eq(user::Column::name, "John")),
    )
    .all(&mut conn)
    .await?;
```

## Operators

### Common (all column wrappers)

- `.eq(value)`
- `.ne(value)`
- `.is_null()`
- `.asc()`, `.desc()` (for ordering, not filtering)

### Numeric (`NumericColumn`)

- `.gt(value)`, `.gte(value)`
- `.lt(value)`, `.lte(value)`
- `.between(min, max)`
- `.in_(vec![...])`

### String (`StringColumn`)

- `.like(pattern)`
- `.not_like(pattern)`
- `.contains(text)`
- `.starts_with(text)`
- `.ends_with(text)`
- `.in_(vec![...])`

### Date-like (`DateLikeColumn`)

- `.gt`, `.gte`, `.lt`, `.lte`, `.between`

## Composing expressions

`WhereClause` supports logical composition:

- `.and(...)`
- `.or(...)`
- `.not_()`

```rust
let clause = user::COLUMN.name.starts_with("A")
    .or(user::COLUMN.name.starts_with("B"))
    .and(user::COLUMN.id.gt(10));
```

Parentheses are emitted to preserve boolean precedence for nested `AND`/`OR` combinations.

## Notes

- `Finder` is immutable-builder style (methods consume `self`).
- If you keep a `Finder` inside another struct and need to reuse it, cloning may be required before applying `offset/limit/filter`.

## Runnable examples

- `crates/corrosion-orm/examples/filtering.rs`
- `crates/corrosion-orm/examples/composed_filtering.rs`
