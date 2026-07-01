# Macro Attributes Reference

This section documents derive/field attributes used by Corrosion ORM macros.

## Model mapping attributes

- [`#[Table]`](table.md)
- [`#[Column]`](column.md)
- [`#[PrimaryKey]`](primary-key.md)
- [`#[Index]`](index.md)
- [`#[HasOne]`](has-one.md)
- [`#[HasMany]`](has-many.md)
- [`#[BelongsTo]`](belongs-to.md)

## Validation attributes

- [`#[NotNull]`](not-null.md)
- [`#[Size]`](size.md)
- [`#[Pattern]`](pattern.md)
- [`#[Email]`](email.md)

## Typical derive usage

```rust
#[derive(Model, Validate)]
#[Table(name = "users")]
#[Index(fields = ["email"], unique = true)]
pub struct User {
    #[PrimaryKey]
    pub id: i64,

    #[NotNull]
    #[Size(min = 3, max = 64)]
    pub username: String,

    #[Email]
    pub email: String,
}
```
