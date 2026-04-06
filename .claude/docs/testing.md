# Testing

## Test Organization

| Type              | Location                                | Runs With           | Purpose                          |
| ----------------- | --------------------------------------- | ------------------- | -------------------------------- |
| Unit tests        | `#[cfg(test)] mod tests` in source file | `cargo nextest run` | Test private logic in isolation  |
| Integration tests | `tests/*.rs` or `tests/*/main.rs`       | `cargo nextest run` | Test public API from the outside |
| Doc-tests         | `/// ``` ... ```` in doc comments       | `cargo test --doc`  | Verify examples compile and run  |

- Unit tests live next to the code they test — no separate `tests/unit/` directory
- Integration tests in `tests/` can only access the crate's public API
- Use `tests/common/mod.rs` (not `tests/common.rs`) for shared helpers — avoids Cargo treating it as a test file
- Doc-tests don't run under nextest; use `cargo test --doc` separately

## Test Style

Test behavior, not implementation. If a refactor breaks tests but not functionality, the tests were wrong.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_input() {
        let result = parse("42");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn parse_rejects_empty_string() {
        let result = parse("");
        assert!(result.is_err());
    }
}
```

- Name tests descriptively: `<action>_<scenario>` or `<action>_<expected_outcome>`
- One assertion per test when practical — failures are easier to diagnose
- Test edges and errors, not just the happy path: empty inputs, boundaries, malformed data
- Don't test private internals unless the logic is complex and the public API is too coarse to exercise it
- Use `assert_eq!` over `assert!` when comparing values — better failure messages

## Async Tests

Use `#[tokio::test]` for async test functions. Prefer the multi-threaded runtime for realistic behavior.

```rust
#[tokio::test]
async fn fetch_returns_data() {
    let result = fetch_data("key").await;
    assert!(result.is_ok());
}

// Single-threaded when you need deterministic ordering
#[tokio::test(flavor = "current_thread")]
async fn sequential_test() {
    // ...
}
```

## Mocking

Mock boundaries, not logic. Only mock things that are slow, non-deterministic, or external.

| Boundary    | Mock Strategy                                                      |
| ----------- | ------------------------------------------------------------------ |
| HTTP APIs   | `wiremock` or `httpmock`                                           |
| Database    | Real DB in tests (testcontainers) or trait abstraction             |
| File system | `tempfile` crate for temp dirs                                     |
| Time        | `tokio::time::pause()` or inject a clock trait                     |
| Randomness  | Accept `impl Rng` parameter, pass `StdRng::seed_from_u64` in tests |

Prefer dependency injection over mocking frameworks:

```rust
// Production: real client
// Test: pass a fake that implements the trait
trait Storage {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
}

fn process(storage: &impl Storage) {
    // ...
}
```

## Property-Based Testing

Use `proptest` for parsers, serialization, algorithms, and anything with a wide input space.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_serialization(value: MyStruct) {
        let bytes = serialize(&value);
        let decoded = deserialize(&bytes)?;
        prop_assert_eq!(value, decoded);
    }

    #[test]
    fn sort_is_idempotent(mut vec: Vec<i32>) {
        vec.sort();
        let sorted = vec.clone();
        vec.sort();
        prop_assert_eq!(vec, sorted);
    }
}
```

- Derive `Arbitrary` for custom types or implement manual strategies
- Use `prop_compose!` for complex generators
- Property tests complement example-based tests — don't replace them

## Snapshot Testing

Use `insta` for testing complex output (serialized structs, error messages, rendered templates).

```rust
use insta::assert_snapshot;

#[test]
fn error_message_format() {
    let err = validate(bad_input);
    assert_snapshot!(err.to_string());
}

#[test]
fn config_serialization() {
    let config = Config::default();
    insta::assert_yaml_snapshot!(config);
}
```

- Run `cargo insta review` to accept/reject snapshot changes
- Commit snapshot files (`*.snap`) to version control
- Use `assert_debug_snapshot!` for Debug output, `assert_yaml_snapshot!` for structured data

## Mutation Testing

Use `cargo-mutants` to verify tests catch real bugs, not just exercise code.

```sh
cargo mutants --package my-crate
```

- Introduces small code mutations (replace `+` with `-`, flip conditions, return defaults)
- If the test suite still passes after a mutation, the test coverage has a gap
- Run periodically, not on every commit — it's slow
- Focus on modules with complex logic, not boilerplate

## Test Fixtures

Use `rstest` for parameterized tests and fixtures:

```rust
use rstest::rstest;

#[rstest]
#[case("hello", 5)]
#[case("", 0)]
#[case("café", 4)]
fn string_length(#[case] input: &str, #[case] expected: usize) {
    assert_eq!(input.len(), expected);
}
```

For shared setup, prefer helper functions over `rstest` fixtures — simpler and more explicit.

## Test Utilities

| Crate            | Purpose                                   |
| ---------------- | ----------------------------------------- |
| `proptest`       | Property-based testing                    |
| `insta`          | Snapshot testing                          |
| `rstest`         | Parameterized tests and fixtures          |
| `wiremock`       | HTTP server mocking                       |
| `tempfile`       | Temporary files and directories           |
| `testcontainers` | Dockerized dependencies (DB, Redis, etc.) |
| `assert_cmd`     | CLI integration testing                   |
| `predicates`     | Assertion helpers for `assert_cmd`        |
| `tokio-test`     | Async test utilities                      |
| `fake`           | Generate fake data (names, emails, etc.)  |

## What Not to Test

- Don't test the compiler (type system already guarantees it)
- Don't test trivial getters/setters
- Don't test third-party crate behavior
- Don't write tests that duplicate the implementation ("test by copy-paste")
