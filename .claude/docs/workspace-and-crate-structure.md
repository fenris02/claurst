# Workspace & Crate Structure

## When to Use a Workspace

Use a workspace when:

- You have a binary and a library (the most common case)
- Multiple crates share dependencies and should use the same versions
- You want to test everything with a single `cargo nextest run --workspace`

Don't create a workspace for a single crate with no plans to split.

## Standard Layout

```
project/
├── Cargo.toml          # Workspace root: members, shared deps, lints
├── rust-toolchain.toml
├── .clippy.toml
├── .rustfmt.toml
├── deny.toml
├── bin/
│   ├── Cargo.toml      # Binary crate
│   └── src/
│       └── main.rs
├── lib/
│   ├── Cargo.toml      # Library crate
│   └── src/
│       ├── lib.rs
│       └── ...
└── tests/              # Workspace-level integration tests (optional)
```

- The workspace `Cargo.toml` should not be a package itself — it only defines `[workspace]`
- Each member has its own `Cargo.toml` with `[package]` and inherits shared settings

## Member Cargo.toml

```toml
[package]
name = "my-lib"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
serde = { workspace = true }

[lints]
workspace = true
```

- Inherit `version`, `edition`, `license`, `rust-version`, `publish` from workspace
- Inherit `[lints]` with `workspace = true` — all members share the same lint config
- Inherit dependencies from `[workspace.dependencies]`

## When to Split Crates

Split when:

- A library has genuinely different consumers (CLI users vs API users)
- Compile times are painful and modules have independent dependency trees
- You need different feature flags for different consumers
- A module is reusable across unrelated projects

Don't split when:

- You're only separating "for organization" — modules within a crate work fine
- The crates would have circular dependencies
- The split creates a trivial crate with one re-export

## Crate Boundaries

| Pattern                          | Example                                                 |
| -------------------------------- | ------------------------------------------------------- |
| `bin/` + `lib/`                  | Binary depends on library; library is the testable core |
| `core/` + `cli/` + `server/`     | Shared core with multiple frontends                     |
| `types/` + `client/` + `server/` | Shared types across client/server                       |
| `macros/` + `lib/`               | Proc-macro crate (must be separate)                     |

## Public API Design Across Crates

- Re-export key types from the top-level crate so consumers don't need to depend on internal crates
- Use `#[doc(inline)]` on re-exports to show documentation at the re-export site
- Don't expose internal crate structure in public paths — use `pub use` to flatten

```rust
// lib/src/lib.rs
mod parser;
mod types;

// Public API — consumers see these at the crate root
#[doc(inline)]
pub use parser::Parser;
#[doc(inline)]
pub use types::{Config, Error, Result};
```

## Module Organization Within a Crate

Prefer flat module structure until complexity demands nesting:

```
src/
├── lib.rs          # Module declarations, re-exports
├── config.rs       # Configuration types
├── error.rs        # Error types
├── parser.rs       # Parser implementation
└── types.rs        # Shared types
```

When a module grows beyond ~500 lines, split into a directory:

```
src/
├── lib.rs
└── parser/
    ├── mod.rs      # Public API, re-exports from submodules
    ├── lexer.rs
    └── ast.rs
```

## Anti-Patterns

| Anti-Pattern                          | Fix                                      |
| ------------------------------------- | ---------------------------------------- |
| Workspace root is also a package      | Separate `[workspace]` from `[package]`  |
| Circular dependencies between members | Merge the crates or extract shared types |
| `pub use *` across crate boundaries   | Explicit re-exports                      |
| One-file crates                       | Keep as a module in the parent crate     |
| Deep nesting (`a::b::c::d::Type`)     | Flatten with `pub use`                   |
| `#[path = "..."]` module overrides    | Standard file layout                     |
