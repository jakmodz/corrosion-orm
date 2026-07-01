# `#[Pattern]`

Validates a string against a regex pattern.

## Targets

- Field on a `#[derive(Validate)]` struct

## Supported types

- `String` only

## Syntax

```rust
#[Pattern(pattern = r"^\\d{10}$")]
```

With custom message:

```rust
#[Pattern(pattern = r"^[a-z0-9_]+$", message = "invalid username")]
```

## Parameters

- `pattern: String` (required)
- `message: Option<String>`

## Notes

- Regex validity is checked during macro parsing.
