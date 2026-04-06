# Feature Flags

## Core Principle

Features must be **additive**. Enabling a feature should only add capabilities, never remove or change existing behavior. Two crates depending on yours with different features enabled must both work when Cargo unifies features.

## Defining Features

```toml
[features]
default = ["json"]
json = ["dep:serde_json"]
yaml = ["dep:serde_yaml"]
tls = ["dep:rustls"]
unstable = []  # Gate experimental APIs
```

- Use `dep:` syntax to make optional dependencies feature-gated without creating implicit features
- Name features after what they enable, not the crate they pull in: `tls` not `rustls`
- Keep `default` minimal — users can't easily un-default features in transitive dependencies
- Use `unstable` or `experimental` features to gate APIs that may change

## Optional Dependencies

```toml
[dependencies]
serde = { version = "1", optional = true }
serde_json = { version = "1", optional = true }

[features]
json = ["dep:serde", "dep:serde_json"]
```

- `dep:` prefix (Rust 1.60+) prevents the crate name from becoming an implicit feature
- Always use `dep:` — implicit features are confusing and can't be removed without breaking changes
- Group related optional dependencies under a single feature

## Conditional Compilation

```rust
#[cfg(feature = "json")]
mod json;

#[cfg(feature = "json")]
pub use json::to_json;

// Conditional trait implementation
#[cfg(feature = "serde")]
impl serde::Serialize for MyType {
    // ...
}

// Conditional derive
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    pub name: String,
}
```

## Feature Hygiene

- **Never use `cfg(not(feature = "..."))`** — it means removing a feature changes behavior, violating additivity
- **Never change function signatures based on features** — use separate functions or trait impls
- **Gate modules, not individual lines** — keep `#[cfg]` at module or item level, not scattered through function bodies
- **Document which features enable which APIs** — use `#[doc(cfg(feature = "..."))]` (nightly) or note it in doc comments

## Testing Feature Combinations

Use `cargo-hack` to verify all feature combinations compile:

```sh
# Test every individual feature
cargo hack check --each-feature --workspace

# Test the full powerset (expensive — use for CI)
cargo hack check --feature-powerset --workspace

# Skip combinations that don't make sense
cargo hack check --feature-powerset --skip unstable
```

- `--each-feature` is fast and catches most issues
- `--feature-powerset` is thorough but exponential — use selectively
- Add to CI for library crates where feature correctness matters

## Common Patterns

### `std` vs `no_std`

For libraries that support `no_std`, use an additive `std` feature:

```toml
[features]
default = ["std"]
std = []
```

```rust
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;
```

### Feature-Gated Re-exports

```rust
// Re-export optional functionality at crate root
#[cfg(feature = "json")]
#[doc(inline)]
pub use json::{from_json, to_json};
```

### Feature Documentation

Document features in the crate-level docs:

```rust
//! # Features
//!
//! - `json` — JSON serialization via `serde_json`
//! - `yaml` — YAML serialization via `serde_yaml`
//! - `tls` — TLS support via `rustls`
```

## Anti-Patterns

| Anti-Pattern                         | Fix                                       |
| ------------------------------------ | ----------------------------------------- |
| `cfg(not(feature = "x"))`            | Make features additive only               |
| Feature changes function return type | Use separate functions                    |
| Implicit feature from optional dep   | Use `dep:` syntax                         |
| Feature enables unsafe code          | Document clearly, consider separate crate |
| `default = ["full"]` with everything | Keep defaults minimal                     |
| Feature named after crate (`serde`)  | Name after capability (`serialization`)   |
