# Corrosion ORM Documentation

Welcome to the docs hub for Corrosion ORM.

## Start here

1. [Getting Started](getting-started.md)
2. [Type Mapping](guides/type-mapping.md)
3. [Relations](guides/relations.md)
4. [Filtering](guides/filtering.md)
5. [Generated `COLUMN` / `Column` modules](guides/column-modules.md)
6. [Validation](guides/validation.md)
7. [Migrations](guides/migrations.md)
8. [Transactions](guides/transactions.md)
9. [Macro Attributes Reference](guides/attributes/README.md)

---

## Guide index

### Core guides

- [Getting Started](getting-started.md)
- [Type Mapping](guides/type-mapping.md)
- [Relations](guides/relations.md)
- [Filtering](guides/filtering.md)
- [Generated `COLUMN` / `Column` modules](guides/column-modules.md)
- [Validation](guides/validation.md)
- [Migrations](guides/migrations.md)
- [Transactions](guides/transactions.md)
- [Macro Attributes Reference](guides/attributes/README.md)
- [Attribute Reference](guides/attributes/README.md)
### Planning / expansion
- Caching
- Better updating(probably some kind of dirty chececking)
- Postgres driver
- Enum support
---

## Examples

Runnable examples live in `crates/corrosion-orm/examples/`:

- `save.rs`
- `find.rs`
- `filtering.rs`
- `composed_filtering.rs`
- `ordering.rs`
- `pagination.rs`
- `cursor_pagination.rs`
- `eager_fetch.rs`
- `lazy_fetch.rs`
- `validation.rs`
- `transaction.rs`
- `delete.rs`

Run an example:

```bash
cargo run --example filtering
```

---

## Notes

- Current dialect support in this repository is SQLite.
- This project is educational/experimental and not production-ready.
