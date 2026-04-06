# Dependencies & Supply Chain

## Selecting Dependencies

- Prefer crates listed on [blessed.rs](https://blessed.rs/crates) when a match exists
- Evaluate before adding: maintenance status, download count, dependency tree size, license compatibility
- Each dependency is attack surface and maintenance burden — justify the addition
- Prefer `std` types at API boundaries; don't leak third-party types in public APIs
- If two crates solve the same problem, prefer the one with fewer transitive dependencies

## Workspace Dependency Inheritance

Declare shared dependencies once in the workspace root, reference them in members:

```toml
# Workspace Cargo.toml
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"

# Member Cargo.toml
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
```

- All version specifications live in `[workspace.dependencies]` — members use `workspace = true`
- Members can add features on top: `serde = { workspace = true, features = ["rc"] }`
- Members cannot override the version — only add features
- `[workspace.package]` fields (`edition`, `license`, `rust-version`) are inherited the same way

## Version Pinning

- Use exact versions in applications: `serde = "=1.0.219"`
- Use semver ranges in libraries: `serde = "1"` — lets consumers resolve compatible versions
- Always run `cargo update` periodically to pick up patch releases
- Pin `Cargo.lock` in version control for binaries and applications
- Do not commit `Cargo.lock` for libraries (let consumers resolve)

## Supply Chain Security

### cargo deny

Primary tool for policy enforcement. Configure in `deny.toml`:

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0"]

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-git = []
```

Run `cargo deny check` before merging dependency changes.

### cargo audit

Checks dependencies against the RustSec advisory database:

```sh
cargo audit
```

- Run in CI on every build
- Subscribe to RustSec advisories for early warning
- Use `cargo audit fix` to auto-update patched versions when available

### cargo vet

For high-assurance supply chain verification:

```sh
cargo vet
```

- Records audit decisions per crate version
- Shares trust across organizations via audit imports
- More rigorous than `cargo deny` — use for security-critical projects

## Updating Dependencies

```sh
# Check for outdated dependencies
cargo outdated

# Update all dependencies within semver constraints
cargo update

# Upgrade to new major versions (requires cargo-edit)
cargo upgrade --incompatible
```

- Review changelogs before major version bumps
- Run the full test suite after any dependency update
- Use `cargo upgrade --dry-run` to preview changes

## Dependency Hygiene

- Remove unused dependencies: `cargo udeps` (nightly) or `cargo machete` (stable)
- Minimize feature flags — don't enable `full` when you need one feature
- Audit `build-dependencies` and `proc-macro` crates — they run at compile time with full system access
- Check transitive dependency count: `cargo tree | wc -l`
- Use `cargo tree -d` to find duplicate versions of the same crate
