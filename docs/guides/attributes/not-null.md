# `#[NotNull]`

Validates that a field is present/non-empty.

## Targets

- Field on a `#[derive(Validate)]` struct

## Syntax

```rust
#[NotNull]
```

or with custom message:

```rust
#[NotNull(message = "username is required")]
```

## Behavior

- For `String`: fails if empty string.
- For `Option<T>`: fails if `None`.
