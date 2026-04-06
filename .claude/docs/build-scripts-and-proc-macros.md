# Build Scripts & Proc Macros

## Build Scripts (build.rs)

### When to Use

| Use Case                 | Example                                                |
| ------------------------ | ------------------------------------------------------ |
| Code generation          | Generating Rust from protobuf, flatbuffers, or schemas |
| Native library linking   | `println!("cargo::rustc-link-lib=sqlite3")`            |
| Compile-time information | Embedding git hash, build timestamp                    |
| Conditional compilation  | Setting `cfg` flags based on target or environment     |

Don't use build scripts for tasks that can be done at runtime or with proc macros.

### Structure

```rust
// build.rs at the crate root
fn main() {
    // Tell Cargo when to re-run this script
    println!("cargo::rerun-if-changed=schema/");
    println!("cargo::rerun-if-env-changed=MY_VAR");

    // Set cfg flags
    println!("cargo::rustc-cfg=has_feature_x");

    // Pass information to the crate
    println!("cargo::rustc-env=BUILD_VERSION={}", env!("CARGO_PKG_VERSION"));
}
```

### Key Rules

- **Always set `rerun-if-changed`** — without it, the build script runs on every build
- **Minimize work** — build scripts run sequentially and block compilation
- **Write generated files to `OUT_DIR`** — never write into `src/`
- **Use `cargo::` prefix** (Rust 1.77+) instead of deprecated `cargo:` for directives
- **Handle errors gracefully** — a panicking build script fails the entire build

### Including Generated Code

```rust
// In your source file
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

### Common Directives

| Directive                         | Purpose                      |
| --------------------------------- | ---------------------------- |
| `cargo::rerun-if-changed=PATH`    | Re-run when file/dir changes |
| `cargo::rerun-if-env-changed=VAR` | Re-run when env var changes  |
| `cargo::rustc-link-lib=NAME`      | Link a native library        |
| `cargo::rustc-link-search=PATH`   | Add to library search path   |
| `cargo::rustc-cfg=KEY`            | Set a `cfg` flag             |
| `cargo::rustc-env=KEY=VALUE`      | Set an environment variable  |
| `cargo::warning=MESSAGE`          | Emit a compiler warning      |

### Build Script Dependencies

Declare build-time-only dependencies in `[build-dependencies]`:

```toml
[build-dependencies]
prost-build = "0.13"
```

- Build dependencies are compiled separately and don't affect the runtime binary
- Keep build dependencies minimal — they add to compile time
- Optimize build dependencies: `[profile.dev.build-override] opt-level = 3`

## Proc Macros

### When to Use

| Justified                                         | Not Justified                       |
| ------------------------------------------------- | ----------------------------------- |
| Deriving trait implementations                    | Simple code that a function handles |
| Compile-time validation of DSLs                   | Anything `macro_rules!` can do      |
| Reducing repetitive boilerplate across many types | One-off code generation             |
| Attribute-driven code transformation              | Making code "look cleaner"          |

Proc macros add compile-time cost, debugging difficulty, and IDE complexity. Justify the tradeoff.

### Crate Structure

Proc macros must live in a separate crate with `proc-macro = true`:

```toml
# macros/Cargo.toml
[package]
name = "my-macros"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

The main crate re-exports the macros:

```rust
// lib/src/lib.rs
pub use my_macros::MyDerive;
```

### Core Dependencies

| Crate         | Purpose                                                            |
| ------------- | ------------------------------------------------------------------ |
| `syn`         | Parse Rust syntax into an AST                                      |
| `quote`       | Generate Rust code from templates                                  |
| `proc-macro2` | Wrapper for `proc_macro` types (needed for testing)                |
| `darling`     | Declarative attribute parsing (reduces boilerplate over raw `syn`) |

### Derive Macro Pattern

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyTrait, attributes(my_attr))]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl MyTrait for #name {
            fn method(&self) -> &str {
                stringify!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}
```

### Best Practices

- **Emit clear error messages** — use `syn::Error` with span information so errors point to the right source location
- **Test with `trybuild`** — compile-fail tests verify error messages
- **Use `darling`** for attribute parsing — it handles validation and error reporting
- **Keep generated code minimal** — delegate to regular functions where possible
- **Support `#[cfg(...)]`** — don't strip or ignore conditional compilation attributes
- **Don't panic** — return `compile_error!()` via `syn::Error::to_compile_error()`

### Testing Proc Macros

```rust
// tests/expand.rs — verify macro expansion
#[test]
fn test_derive() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass/*.rs");
    t.compile_fail("tests/cases/fail/*.rs");
}
```

Use `cargo expand` to inspect what a macro generates:

```sh
cargo expand --package my-crate --lib
```

### `macro_rules!` vs Proc Macros

| Feature          | `macro_rules!` | Proc Macro  |
| ---------------- | -------------- | ----------- |
| Compile cost     | Minimal        | Significant |
| Debuggability    | Moderate       | Harder      |
| IDE support      | Good           | Variable    |
| Pattern matching | Built-in       | Via `syn`   |
| Code generation  | Limited        | Unlimited   |
| Separate crate   | No             | Yes         |

Prefer `macro_rules!` when it can do the job. Escalate to proc macros only when you need to inspect type structure, generate impls, or transform attributes.
