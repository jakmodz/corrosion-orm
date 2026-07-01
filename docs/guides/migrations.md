# Migrations

Migrations track schema changes over time in a reproducible way.

## 1) Install the CLI scaffold tool

```bash
cargo install corrosion-orm-cli
```

> This tool currently scaffolds a migration project. The actual migration commands (`generate`, `create`, `up`, `down`, `status`) are run inside that generated project.

## 2) Initialize a migrations project

Default path (`./migrations`):

```bash
corrosion-orm-cli migration init
```

Custom path (positional argument):

```bash
corrosion-orm-cli migration init path/to/migrations
```

This creates a standalone Rust crate for migrations.

## 3) Configure generated dependencies (important)

After init, open `migrations/Cargo.toml`.

If it contains placeholder path dependencies like:

```toml
corrosion-orm = { path = "", features = ["sqlite"] }
corrosion-orm-migration = { path = "" }
```

replace them with published versions, for example:

```toml
corrosion-orm = { version = "0.1.1", features = ["sqlite"] }
corrosion-orm-migration = { version = "0.1.1" }
```

## 4) Register app models

In `migrations/src/main.rs`, implement your `AppRegistry` by registering all models that define your schema:

```rust
pub struct MyAppRegistry;

impl AppRegistry for MyAppRegistry {
    fn app_registry() -> ModelRegistry {
        ModelRegistry::new()
            .register_model::<MyModel>()
            .register_model::<AnotherModel>()
    }
}
```

## 5) Generate migration files

From now on, run commands from inside the migrations crate directory:

```bash
cd migrations
```

### Auto name

```bash
cargo run -- generate
```

### Custom name

```bash
cargo run -- create --name add_users_table
```

## 6) Apply / rollback migrations

`up`, `down`, and `status` require `--database-url`.

### Apply all pending migrations

```bash
cargo run -- --database-url sqlite:../app.db up
```

### Rollback last migration

```bash
cargo run -- --database-url sqlite:../app.db down
```

### Apply/rollback limited number of steps

```bash
cargo run -- --database-url sqlite:../app.db up --steps 2
cargo run -- --database-url sqlite:../app.db down --steps 1
```

### Show migration status

```bash
cargo run -- --database-url sqlite:../app.db status
```

## How migration generation works (snapshot/diff)

When you run `generate` or `create --name ...`, the migration crate:

1. Builds the **current model snapshot** from `AppRegistry`.
2. Reads the **previous snapshot** from `.corrosion/<last_migration>_snapshot.json` (if any).
3. Computes a schema diff (`old -> new`).
4. Writes a new Rust migration file plus a new snapshot JSON.

This means migration generation is deterministic for a given model set and previous snapshot.

## Command reference

Inside the migrations crate:

- `generate`
- `create --name <name>`
- `up [--steps <n>]`
- `down [--steps <n>]`
- `status`

Global option:

- `--database-url <url>` (required for `up`, `down`, `status`)

## Notes

- Your migration snapshots are stored under `.corrosion/`.
- Keep migration files committed to git.
- Prefer descriptive names for `create --name ...` to improve history readability.
