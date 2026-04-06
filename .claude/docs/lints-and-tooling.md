# Lints & Tooling

## Linting Rules

- Keep code clippy-clean: `cargo +nightly clippy --all-targets --all-features -- -D warnings`
- Use `#[expect(clippy::...)]` over `#[allow(clippy::...)]` — `expect` warns when the lint no longer fires
- `#[allow(...)]` is denied — use `#[expect(...)]` instead
- Add a `reason` to lint overrides: `#[expect(clippy::unused_async, reason = "will use I/O later")]`
- Format Rust with `cargo +nightly fmt` — no exceptions
- Format TOML with `taplo format` — validates and sorts keys in `Cargo.toml`, `.clippy.toml`, etc.
- Format Markdown with `dprint fmt` — use for `README.md`, `CHANGELOG.md`, and docs
- Prefer `fd` over `find` and `rg` over `grep` — both are faster, respect `.gitignore`, and have better defaults

## Cargo.toml Lints

```toml
[lints.rust]
unsafe_code = "forbid"
unexpected_cfgs = "warn"
ambiguous_negative_literals = "warn"
missing_debug_implementations = "warn"
redundant_imports = "warn"
redundant_lifetimes = "warn"
trivial_numeric_casts = "warn"
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"

[lints.clippy]
# Major categories
cargo = { level = "warn", priority = -1 }
complexity = { level = "warn", priority = -1 }
correctness = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
perf = { level = "warn", priority = -1 }
style = { level = "warn", priority = -1 }
suspicious = { level = "warn", priority = -1 }

# Panic prevention
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
panic_in_result_fn = "deny"
unimplemented = "deny"

# No cheating
allow_attributes = "deny"
allow_attributes_without_reason = "warn"

# Code hygiene
dbg_macro = "deny"
todo = "deny"
print_stdout = "deny"
print_stderr = "deny"

# Safety
await_holding_lock = "deny"
large_futures = "deny"
exit = "deny"
mem_forget = "deny"
undocumented_unsafe_blocks = "warn"
unnecessary_safety_comment = "warn"
unnecessary_safety_doc = "warn"

# Disallowed items (configured in .clippy.toml)
disallowed_methods = "deny"
disallowed_types = "deny"
disallowed_macros = "deny"

# Restriction lints for consistency
clone_on_ref_ptr = "warn"
empty_drop = "warn"
empty_enum_variants_with_brackets = "warn"
empty_structs_with_brackets = "warn"
map_err_ignore = "warn"
redundant_type_annotations = "warn"
renamed_function_params = "warn"
implicit_clone = "warn"
unused_result_ok = "warn"

# Pedantic relaxations (too noisy)
module_name_repetitions = "allow"
similar_names = "allow"
```

## Clippy Configuration (.clippy.toml)

Configuration lives in `.clippy.toml` at the project root. If a project has `clippy.toml` (without the leading dot), rename it to `.clippy.toml` — the dotfile takes precedence and keeps the root clean.

```toml
# .clippy.toml
msrv = "1.91.0"
avoid-breaking-exported-api = true
check-private-items = true
cognitive-complexity-threshold = 25

# SAFETY comment placement
accept-comment-above-statement = true
accept-comment-above-attributes = true
```

### Disallowed Methods

| Method                          | Reason                  | Use Instead               |
| ------------------------------- | ----------------------- | ------------------------- |
| `std::mem::uninitialized`       | UB                      | `MaybeUninit<T>`          |
| `std::mem::zeroed`              | UB for non-POD types    | `MaybeUninit<T>`          |
| `f32::abs_sub` / `f64::abs_sub` | Deprecated              | `(a - b).abs()`           |
| `std::env::set_var`             | Unsafe in Edition 2024  | Wrap in `unsafe {}`       |
| `std::env::remove_var`          | Unsafe in Edition 2024  | Wrap in `unsafe {}`       |
| `std::process::Command::new`    | CWD injection risk      | Use absolute paths        |
| `std::fs::canonicalize`         | UNC paths on Windows    | `dunce::canonicalize`     |
| `libc::strtok`                  | Not reentrant           | `str::split`              |
| `libc::localtime`               | Not reentrant           | `chrono` or `localtime_r` |
| `libc::strerror`                | Not reentrant           | `std::io::Error`          |
| `libc::getenv`                  | Inherent race condition | `std::env::var`           |

### Disallowed Types

| Type                 | Reason            | Use Instead          |
| -------------------- | ----------------- | -------------------- |
| `std::sync::mpsc::*` | Known perf issues | `crossbeam::channel` |

### Disallowed Macros

| Macro                      | Reason     | Use Instead                        |
| -------------------------- | ---------- | ---------------------------------- |
| `lazy_static::lazy_static` | Superseded | `std::sync::OnceLock` / `LazyLock` |

## Key Clippy Lint Mapping

| Lint                         | Fix                        |
| ---------------------------- | -------------------------- |
| `unwrap_used`                | Use `?` or `expect()`      |
| `needless_clone`             | Use reference              |
| `await_holding_lock`         | Scope guard before await   |
| `linkedlist`                 | Use `Vec`/`VecDeque`       |
| `wildcard_imports`           | Explicit imports           |
| `missing_safety_doc`         | Add `# Safety` doc section |
| `undocumented_unsafe_blocks` | Add `// SAFETY:` comment   |
| `transmute_ptr_to_ptr`       | Use `pointer::cast()`      |
| `large_stack_arrays`         | Use `Vec` or `Box`         |
| `too_many_arguments`         | Use struct params          |

## Toolchain (rust-toolchain.toml)

Pin the toolchain at the project root. Create `rust-toolchain.toml` if it doesn't exist.

```toml
[toolchain]
channel = "stable"
components = ["clippy", "llvm-tools", "rustfmt"]
profile = "minimal"
```

- `profile = "minimal"` avoids downloading docs and other extras
- Add `"llvm-tools"` for coverage (`cargo-llvm-cov`) and profiling
- Always use `cargo +nightly fmt` and `cargo +nightly clippy` to get the full set of lints and formatting options

## Static Verification Tools

| Tool                    | Purpose                                             |
| ----------------------- | --------------------------------------------------- |
| `cargo +nightly clippy` | Lint and code quality (Rust)                        |
| `cargo +nightly fmt`    | Consistent formatting (Rust)                        |
| `taplo format`          | Format and lint TOML files                          |
| `dprint fmt`            | Format Markdown files                               |
| `cargo nextest run`     | Test runner (preferred over `cargo test`)           |
| `cargo-edit`            | `cargo add`, `cargo rm`, `cargo upgrade`            |
| `cargo-audit`           | Security vulnerability check                        |
| `cargo-hack`            | Feature combination validation                      |
| `cargo-udeps`           | Unused dependency detection                         |
| `cargo deny`            | Advisories, licenses, bans                          |
| `miri`                  | Unsafe code validation                              |
| `cargo careful test`    | Stdlib debug assertions + UB checks                 |
| `mdbook build`          | Build project documentation book                    |
| `fd`                    | File finder (preferred over `find`)                 |
| `rg`                    | Ripgrep — fast regex search (preferred over `grep`) |

Prefer `cargo nextest run` over `cargo test` — it runs tests in parallel per-test (not per-crate), gives better output on failure, and supports retries. Fall back to `cargo test` only for doc-tests, which nextest does not support.

## Tool Installation

Install the nightly toolchain and Miri component via rustup. Install all other tools via `cargo install`. Always look up the current stable version before installing — never assume from memory.

```sh
# Nightly toolchain (required for fmt and clippy)
rustup toolchain install nightly --profile minimal
rustup component add miri --toolchain nightly

# CLI utilities
cargo install fd-find --locked
cargo install ripgrep --locked

# Core workflow
cargo install cargo-nextest --locked
cargo install cargo-edit --locked
cargo install cargo-deny --locked
cargo install cargo-audit --locked
cargo install taplo-cli --locked
cargo install dprint --locked

# Extended verification
cargo install cargo-hack --locked
cargo install cargo-udeps --locked
cargo install cargo-careful --locked
cargo install cargo-mutants --locked

# Documentation
cargo install mdbook --locked

# Coverage
cargo install cargo-llvm-cov --locked
```

Clippy, rustfmt, and llvm-tools are installed via `rust-toolchain.toml` components — no separate install needed.

## Benchmarking

Enable debug symbols for meaningful profiling:

```toml
[profile.bench]
debug = 1
```

Recommended tools: `criterion` or `divan`.
