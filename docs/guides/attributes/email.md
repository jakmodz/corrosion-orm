# `#[Email]`

Validates string as email format.

## Targets

- Field on a `#[derive(Validate)]` struct

## Supported types

- `String` only

## Syntax

```rust
#[Email]
```

or

```rust
#[Email(message = "invalid email")]
```

## Parameters

- `message: Option<String>`

## Notes

- Uses built-in email regex in generated validator code.
