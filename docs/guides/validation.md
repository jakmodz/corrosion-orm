# Validation

Corrosion ORM supports declarative validation via `#[derive(Validate)]`.

## Supported attributes

- `#[NotNull]`
- `#[Size(min = ..., max = ...)]`
- `#[Pattern(pattern = "...")]`
- `#[Email]`

Each validation attribute also supports optional `message = "..."`.

## Basic usage

```rust
use corrosion_orm::Validate;

#[derive(Validate)]
struct SignupForm {
    #[NotNull]
    #[Size(min = 3, max = 20)]
    username: String,

    #[Email]
    email: String,

    #[Pattern(pattern = r"^\d{10}$")]
    phone: String,
}

let form = SignupForm { /* ... */ };
form.validate()?;
```

## Validation on models

You can derive both `Model` and `Validate`:

```rust
#[derive(Model, Validate)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,

    #[NotNull]
    #[Size(min = 1)]
    pub name: String,

    #[Email]
    pub email: String,
}
```

Typical flow:

```rust
user.validate()?;
user.save(&mut conn).await?;
```

## Behavior details

1. Validation returns on the **first** failing rule.
2. Field validations run in declaration order.
3. `#[Size]`, `#[Pattern]`, and `#[Email]` are currently restricted to `String` fields.
4. `#[Pattern]` regex is validated at compile time by the macro parser.

## Error types

Validation errors are represented by `ValidationError` and include:

- null/empty field
- size errors
- pattern mismatch
- custom message error

## Runnable example

- `crates/corrosion-orm/examples/validation.rs`
