# Error Handling

## Core Principles

- Use `Result<T, E>` for expected failures, `Option<T>` for absent values
- `panic!` only for invariant violations and detected programming bugs — not for error communication
- Panics are not exceptions; they mean "stop the program"
- If encountering a programming error (contract violation), panic — don't return an `Error` that can't be acted upon

## Library vs Application

| Context          | Approach                                               |
| ---------------- | ------------------------------------------------------ |
| Library code     | `thiserror` for custom error enums with `#[error]`     |
| Application code | `anyhow::Result` with `.context("what was happening")` |

- Don't mix multiple application-level error crates; pick one and use it consistently
- Libraries must not use `anyhow` in their public API

## Error Propagation

- Always add context when propagating errors — bare `?` loses "what was happening"
- Use `_else` variants (`unwrap_or_else`, `ok_or_else`) when the fallback allocates or computes
- Chain `inspect_err` (log) and `map_err` (transform) for clean error pipelines
- Never use `unwrap()` in production paths — propagate with `?`
- Use `expect()` over `unwrap()` only when the value is genuinely guaranteed
- Use `assert!` for invariant checks at function entry

## Option & Result Patterns

| Pattern    | Use When                                                   |
| ---------- | ---------------------------------------------------------- |
| `match`    | Pattern matching against inner variants                    |
| `let-else` | Divergent branch is simple (`return`, `continue`, `break`) |
| `if let`   | Non-match branch needs computation                         |
| `?`        | Propagate error to caller                                  |

## Deprecated Patterns

| Deprecated                | Use Instead            |
| ------------------------- | ---------------------- |
| `failure` / `error-chain` | `thiserror` / `anyhow` |
| `try!()`                  | `?` operator           |
