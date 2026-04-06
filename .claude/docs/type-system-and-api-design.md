# Type System & API Design

## Type-Driven Design

- Use newtype wrappers to prevent parameter mixups at compile time (e.g., `struct AccountId(i64)` not bare `i64`)
- Use the typestate pattern to encode state machines — invalid transitions become compile errors
- Prefer enums or options structs over boolean parameters
- Make invalid states unrepresentable — avoid `Option<Option<T>>`, use custom enums with clear semantics
- Validate once at construction, trust the type thereafter ("parse, don't validate")
- No wildcard matches (`_` catch-all); avoid `matches!` macro — explicit destructuring catches field additions at compile time
- Use arrays for fixed sizes; use `Vec::with_capacity()` / `String::with_capacity()` for pre-allocation
- Prefer slice patterns: `if let [first, .., last] = slice`

## API Design

- Accept `&str` or `impl AsRef<str>` not `String` in function parameters
- Accept `impl AsRef<Path>` for file paths, use `PathBuf`/`Path` not `String`
- Accept `impl Read`/`impl Write` for I/O (sans-io pattern)
- Return `&str` not `&String` from accessors
- Prefer `Option<&T>` over `&Option<T>` in public APIs
- Avoid exposing `Arc`/`Rc`/`Box`/`RefCell` in public APIs — hide behind clean `&T`/`&mut T`/`T` interfaces
- Use builder pattern when 3+ optional parameters
- Use `#[must_use]` for important return values
- Return owned data from constructors: `fn new(name: impl Into<String>) -> Self`
- Public types should implement `Debug` (custom impl for sensitive data); implement `Display` for types meant to be read
- Eagerly implement common traits: `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Default`
- Have `Foo::new()` even if you have `Foo::default()`

## Generics & Trait Objects

| Preference                 | When to Use                                                            |
| -------------------------- | ---------------------------------------------------------------------- |
| Concrete types             | Default choice                                                         |
| Generics (static dispatch) | Performance-critical, multiple callers need different types            |
| `dyn Trait`                | Heterogeneous collections, plugin architectures, reducing compile time |
| `enum`                     | Set of types is small and known                                        |

- Keep trait bounds narrow — put them on the `impl` that needs them, not the struct
- Prefer regular functions over associated functions for general-purpose computation not tied to a receiver

## Dependencies

- Prefer crates listed on https://blessed.rs/crates when a match exists
- Don't leak third-party types in public APIs; prefer `std` types at boundaries
- Features must be additive; don't introduce `no-std` features (use `std` feature instead)
- If in doubt, split into separate crates for compile time and modularity

## Anti-Patterns to Avoid

- `String` parameters → `&str` or `impl AsRef<str>`
- Stringly-typed APIs → newtype wrappers
- Boolean parameters → options struct or enum
- `Box<T>` returns unnecessarily → return `T` directly
- `pub use foo::*` → explicit re-exports
- `transmute` for enum casts → `match` or `TryFrom`
- OOP via `Deref` → composition
- Over-generic everything → concrete types when possible
- Macros for simple operations → plain functions
- Weasel-word type names (`Service`, `Manager`, `Factory`) → specific names (`BookingDispatcher`, `Builder`)
