# Style & Conventions

## Naming

| Convention  | Rule                                                          | Example                         |
| ----------- | ------------------------------------------------------------- | ------------------------------- |
| Accessors   | No `get_` prefix — bare nouns                                 | `fn name(&self)`                |
| `as_`       | Cheap borrow-to-borrow                                        | `fn as_str(&self) -> &str`      |
| `to_`       | Expensive conversion (allocates)                              | `fn to_string(&self) -> String` |
| `into_`     | Ownership-consuming conversion                                | `fn into_inner(self) -> T`      |
| Iterators   | `iter()` / `iter_mut()` / `into_iter()`                       | Borrow / mut borrow / consume   |
| Cases       | `snake_case` fn/var, `CamelCase` type, `SCREAMING_CASE` const | Standard Rust                   |
| Static vars | `G_CONFIG` prefix for `static`, no prefix for `const`         |                                 |

## Formatting

Format with `cargo +nightly fmt` — no exceptions. Configuration lives in `.rustfmt.toml` at the project root. Always use `+nightly` to get the full set of formatting options (`imports_granularity`, `group_imports`, `wrap_comments`, `format_code_in_doc_comments`, `reorder_impl_items`, `fn_params_layout`).

**Setup:** If the project has `rustfmt` or `rustfmt.toml` (without the leading dot), rename it to `.rustfmt.toml`. If neither exists, create `.rustfmt.toml`. The dotfile form is preferred — it keeps the project root clean and is the conventional name.

```toml
# .rustfmt.toml — Edition 2024 style
unstable_features = true
edition = "2024"
style_edition = "2024"

# Layout
max_width = 100
indent_style = "Block"
use_small_heuristics = "Default"
trailing_comma = "Vertical"
fn_params_layout = "Compressed"
blank_lines_lower_bound = 0
blank_lines_upper_bound = 1
newline_style = "Unix"

# Comments
comment_width = 100
wrap_comments = true
format_code_in_doc_comments = true

# Imports
reorder_imports = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"

# Code style
use_try_shorthand = true
use_field_init_shorthand = true
normalize_doc_attributes = true
reorder_impl_items = true

# Ignore vendored/generated code
ignore = ["/build/", "/*-build/", "/build-*/", "/vendor/"]
```

## Code Style

- Use `let...else` for early returns; keep the happy path unindented
- Use `tracing` for logging (`error!`/`warn!`/`info!`/`debug!`), not `println!`
- Prefer `for` loops with mutable accumulators when clearer than iterator chains
- Use structured logging with message templates and named properties
- Follow OpenTelemetry semantic conventions for log attributes
- Name events with hierarchical dot-notation: `<component>.<operation>.<state>`
- Redact sensitive data in logs

## File Organization

1. Module declarations (`mod`)
2. `use` statements for imports
3. `pub use` for re-exports (annotate with `#[doc(inline)]` for own types)
4. Constants and static variables
5. Type aliases
6. Struct definitions
7. Enum definitions
8. Trait definitions
9. Impl blocks
10. Functions
11. Tests (`#[cfg(test)]` module)

## Import Ordering

Order: `std` / `core` / `alloc` → external crates → workspace crates → `super::` → `crate::`. One import per line.

## Documentation

- Summary sentence < 15 words, on one line
- Extended docs explain parameters in prose (no parameter tables)
- Required sections when applicable: `# Examples`, `# Errors`, `# Panics`, `# Safety`, `# Abort`
- Module-level `//!` documentation for all public library modules
- Use `///` for public items, `//!` for module docs
- Magic values must have comments explaining why, side effects, and external interactions; prefer named constants

## Deprecated Patterns

| Deprecated     | Use Instead                       |
| -------------- | --------------------------------- |
| `extern crate` | Just `use`                        |
| `#[macro_use]` | Explicit `use crate::macro_name!` |
