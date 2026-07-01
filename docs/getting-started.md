# Getting Started

This guide helps you get productive quickly with Corrosion ORM.

> Corrosion ORM is currently a learning project and not production-ready.

## 1) Add dependency

```toml
[dependencies]
corrosion-orm = "0.1.1"
tokio = { version = "1", features = ["full"] }
```

## 2) Define a model

```rust
use corrosion_orm::Model;
use corrosion_orm::prelude::*;

#[derive(Model, Debug)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,
    pub name: String,
}
```

## 3) Connect and create schema

```rust
let db = corrosion_orm::connect(":memory:").await?;
let mut conn = db.acquire_conn().await?;

let mut ctx = QueryContext::from_model(User::get_schema(), conn.get_dialect());
conn.execute_query(&mut ctx).await?;
```

## 4) Run basic query

```rust
let users = User::find()
    .filter(user::COLUMN.id.gt(10))
    .all(&mut conn)
    .await?;
```

---
# Crate features

| Feature | Description |
|---------|-------------|
| `sqlite` | Sqlite backend driver |
| `log`    | Logging for queries   |

# Core Guides

- [Type Mapping](guides/type-mapping.md)
- [Relations](guides/relations.md)
- [Filtering](guides/filtering.md)
- [Generated `COLUMN` / `Column` Modules](guides/column-modules.md)
- [Validation](guides/validation.md)
- [Migrations](guides/migrations.md)
- [Transactions](guides/transactions.md)
- [Macro Attributes Reference](guides/attributes/README.md)

# Suggested Additional Documentation

- [Documentation Roadmap (recommended topics)](guides/documentation-roadmap.md)

# Full runnable examples

See `crates/corrosion-orm/examples/`:

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
