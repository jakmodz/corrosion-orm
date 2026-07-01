# `#[Size]`

Validates string length bounds.

## Targets

- Field on a `#[derive(Validate)]` struct

## Supported types

- `String` only

## Syntax

```rust
#[Size(min = 3, max = 20)]
```

With custom message:

```rust
#[Size(min = 3, message = "too short")]
```

## Parameters

- `min: Option<usize>`
- `max: Option<usize>`
- `message: Option<String>`
