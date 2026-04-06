# CI Pipeline

## Pipeline Stages

Run checks in order of speed — fail fast on the cheapest checks first:

```
1. Format check     (seconds)    — cargo +nightly fmt -- --check
2. TOML check       (seconds)    — taplo check
3. Markdown check   (seconds)    — dprint check
4. Compile check    (fast)       — cargo check --workspace --all-targets
5. Clippy           (moderate)   — cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings
6. Unit/integration (moderate)   — cargo nextest run --workspace
7. Doc-tests        (moderate)   — cargo test --doc --workspace
8. Supply chain     (fast)       — cargo deny check
9. Audit            (fast)       — cargo audit
10. Coverage        (slow)       — cargo llvm-cov nextest (optional, on merge only)
```

## Minimal CI Configuration

```yaml
# Stages run sequentially; steps within a stage run in parallel where possible

check:
  - cargo +nightly fmt -- --check
  - taplo check
  - cargo check --workspace --all-targets

lint:
  - cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings
  - cargo deny check
  - cargo audit

test:
  - cargo nextest run --workspace
  - cargo test --doc --workspace
```

## CI Best Practices

- **Cache `~/.cargo/registry` and `target/`** to avoid re-downloading and recompiling dependencies
- **Pin toolchain** via `rust-toolchain.toml` — CI uses the same version as developers
- **Run on all targets**: `--all-targets` catches issues in tests, benches, and examples
- **Separate formatting from linting** — formatting failures are trivial to fix and should not block the lint stage
- **Run `cargo deny` on dependency changes** — catches license violations and advisories early
- **Use `--locked`** in CI to ensure `Cargo.lock` is up to date and reproducible
- **Fail on warnings** — use `-- -D warnings` with clippy; set `RUSTFLAGS="-D warnings"` for compiler warnings

## Environment Variables

```sh
# Treat all warnings as errors
RUSTFLAGS="-D warnings"

# Backtrace on panic (useful for test failures)
RUST_BACKTRACE=1

# Log level for tests
RUST_LOG=info

# Nextest profile for CI (retries, output format)
NEXTEST_PROFILE=ci
```

## Nextest CI Profile

Create `.config/nextest.toml` for CI-specific behavior:

```toml
[profile.ci]
retries = 2
fail-fast = false
status-level = "all"
final-status-level = "fail"
```

## Optional CI Steps

| Step             | When                    | Command                                                             |
| ---------------- | ----------------------- | ------------------------------------------------------------------- |
| Coverage         | On merge to main        | `cargo llvm-cov nextest --workspace --lcov --output-path lcov.info` |
| MSRV check       | On dependency changes   | `cargo msrv verify`                                                 |
| Semver check     | On library releases     | `cargo semver-checks`                                               |
| Feature combos   | On feature flag changes | `cargo hack check --feature-powerset`                               |
| Unused deps      | Periodic                | `cargo +nightly udeps --workspace`                                  |
| Mutation testing | Periodic                | `cargo mutants`                                                     |
| Careful tests    | Periodic                | `cargo careful test`                                                |

## Branch Protection

- Require all CI checks to pass before merge
- Require up-to-date branch (rebased on main)
- Block force pushes to main/master
- Require at least one approval for PRs
