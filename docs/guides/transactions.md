# Transactions

This guide explains how transactions behave in Corrosion ORM.

## Quick example

```rust
let config = SqliteConfigBuilder::new()
    .url("sqlite:app.db".to_string())
    .max_connections(2)
    .build();

let db = SqliteDriver::new(config).await?;
let mut conn = db.acquire_conn().await?;
let mut tx = db.transaction().await?;

user.save(&mut tx).await?;

// Not visible on a different executor/connection before commit.
assert!(User::get_by_id(1, &mut conn).await?.is_none());

tx.commit().await?;
assert!(User::get_by_id(1, &mut conn).await?.is_some());
```

## Execution model

`db.transaction().await?` creates a `Transaction<P>` wrapper that implements `Executor`.

That means all regular repository/query APIs work in a transaction exactly like with a normal connection:

- `save(&mut tx)`
- `find().all(&mut tx)`
- `get_by_id(..., &mut tx)`

## Visibility rules

- Changes made inside a transaction are visible to queries run through that same transaction executor.
- Queries run through a different connection/executor do not see uncommitted changes.
- After `commit()`, changes are visible to other connections.

## Commit / rollback semantics

- `commit(self)` consumes the transaction and sends `COMMIT`.
- `rollback(self)` consumes the transaction and sends `ROLLBACK`.
- If dropped without explicit commit/rollback, transaction `Drop` triggers rollback as safety fallback.

## SQLite-specific caveats

### 1) `:memory:` and multiple connections

SQLite in-memory databases are per-connection by default. Since transaction and non-transaction queries can run on different pooled connections, this can produce confusing behavior.

For transaction examples, prefer a file-backed database URL like `sqlite:app.db`.

### 2) Pool size matters

If you hold a regular connection and also start a transaction at the same time, you typically need at least 2 pool connections.

Corrosion's `connect(...)` helper uses default pool settings. For explicit transaction demos, configuring `SqliteConfigBuilder` directly is often clearer.

## Recommended pattern

1. Create and use `tx` for all operations that must be atomic.
2. Validate expected state inside transaction if needed.
3. Commit once all operations succeed.
4. On any error path, rollback (or let drop rollback).

## Anti-patterns

- Mixing `&mut tx` writes with reads from a separate `&mut conn` and expecting uncommitted visibility.
- Using `:memory:` with multi-connection assumptions.
- Forgetting to commit and expecting persisted changes.

## Reference

Runnable example: `crates/corrosion-orm/examples/transaction.rs`
