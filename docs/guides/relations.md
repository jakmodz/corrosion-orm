# Relations

Corrosion ORM supports:

- `#[HasOne]`
- `#[HasMany]`
- `#[BelongsTo]`

## Relation matrix

| Attribute | Typical field type (eager) | Lazy field type | Owns FK column physically? | DDL effect on current table |
|---|---|---|---|---|
| `#[HasOne]` | `Related` | `Lazy<Related>` | Yes | Adds relation column + `FOREIGN KEY`, and relation is marked unique (1:1). |
| `#[BelongsTo]` | `Related` | `Lazy<Related>` | Yes | Adds relation column + `FOREIGN KEY`. |
| `#[HasMany]` | `Vec<Related>` | `LazyCollection<Related, related::Column>` | No (FK is on child table) | No FK column added on owner table. |

## Explicit foreign key rule (important)

For type-safe querying, tables that physically store a FK should also declare that FK column as a normal field.

Example (`BelongsTo` side):

```rust
#[derive(Model)]
#[Table(name = "posts")]
pub struct Post {
    #[PrimaryKey]
    pub id: i64,

    #[Column(name = "teacher_id")]
    pub teacher_id: i64,

    #[BelongsTo(foreign_key = "teacher_id", table = "teachers")]
    pub teacher: Teacher,
}
```

Without that explicit field, you lose convenient typed filtering like:

```rust
Post::find().filter(post::COLUMN.teacher_id.eq(1))
```

## Eager vs Lazy

- **Eager**
  - relation field is concrete (`Author`, `Vec<Post>`, etc.)
  - fetched object graph is immediately populated
- **Lazy**
  - relation field is `Lazy<T>` or `LazyCollection<T, _>`
  - call `.load(&mut conn).await?` when needed
  - loaded value is cached in-memory for later accesses

## Cascade behavior

Relation attributes support `cascade` (default: `true`).

- For eager relations, `save()` can cascade.
- For lazy collections (`LazyCollection`), generated save does **not** automatically persist pending children; load/save children explicitly when needed.

## Runnable examples

- Eager: `crates/corrosion-orm/examples/eager_fetch.rs`
- Lazy: `crates/corrosion-orm/examples/lazy_fetch.rs`
