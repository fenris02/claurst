# Versioning & Release

## Semantic Versioning

Follow [semver](https://semver.org/) strictly for library crates:

| Version Bump              | When                                   |
| ------------------------- | -------------------------------------- |
| **Major** (1.x → 2.0)     | Breaking changes to public API         |
| **Minor** (1.0 → 1.1)     | New functionality, backward compatible |
| **Patch** (1.0.0 → 1.0.1) | Bug fixes, no API changes              |

### What Counts as Breaking

| Change                                          | Breaking?                  |
| ----------------------------------------------- | -------------------------- |
| Remove public function/type/module              | Yes                        |
| Change function signature                       | Yes                        |
| Add required field to public struct             | Yes                        |
| Tighten generic bounds                          | Yes                        |
| Bump MSRV                                       | Yes (for libraries)        |
| Remove a feature flag                           | Yes                        |
| Change behavior without API change              | Depends — document clearly |
| Add new public function                         | No                         |
| Add optional field with `#[serde(default)]`     | No                         |
| Add new enum variant (with `#[non_exhaustive]`) | No                         |
| Loosen generic bounds                           | No                         |
| Add a feature flag                              | No                         |

### Semver Checking

Use `cargo-semver-checks` to catch accidental breaking changes before release:

```sh
cargo semver-checks
```

- Compares the current crate against the last published version
- Catches removed items, changed signatures, tightened bounds
- Run in CI on library crates before publishing

## MSRV Policy

The Minimum Supported Rust Version (MSRV) is declared in `Cargo.toml`:

```toml
[workspace.package]
rust-version = "1.91.0"
```

And mirrored in `.clippy.toml`:

```toml
msrv = "1.91.0"
```

### When to Bump MSRV

- When a dependency requires a newer Rust version
- When a new language feature significantly simplifies the code
- When the current MSRV is more than 6 months behind stable (for applications)

### MSRV Rules

- **Libraries**: bumping MSRV is a breaking change — bump the minor or major version
- **Applications**: MSRV bumps are not breaking — update freely
- **Always update both** `Cargo.toml` `rust-version` and `.clippy.toml` `msrv` together
- Verify with `cargo msrv verify` or by testing with the pinned MSRV toolchain:

```sh
cargo +1.91.0 check --workspace
```

## Version Management

### Workspace Versioning

All workspace members share the version from `[workspace.package]`:

```toml
# Workspace root
[workspace.package]
version = "0.1.0"

# Member
[package]
version.workspace = true
```

- Bump the workspace version in one place
- Use `cargo bump` for automated version bumps: `cargo bump minor`

### Pre-release Versions

For unstable APIs, use `0.x` versioning:

| Version | Meaning                                 |
| ------- | --------------------------------------- |
| `0.0.x` | Experimental, anything can change       |
| `0.x.y` | API taking shape, minor bumps may break |
| `1.0.0` | Stable API, semver guarantees begin     |

## Release Checklist

1. Update version number (`cargo bump minor` or edit `Cargo.toml`)
2. Update CHANGELOG.md
3. Run full CI pipeline (format, lint, test, deny, audit)
4. Run `cargo semver-checks` (for libraries)
5. Verify MSRV: `cargo +<msrv> check --workspace`
6. Create a git tag: `git tag -a v0.2.0 -m "v0.2.0"`
7. **Get user confirmation before publishing**
8. Publish: `cargo publish --workspace` (Rust 1.84+)

## Changelogs

Maintain a `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
# Changelog

## [Unreleased]

## [0.2.0] - 2026-04-03

### Added

- New `parse_strict` function for validated input

### Changed

- `Config::new` now returns `Result` instead of panicking

### Fixed

- Off-by-one error in boundary detection
```

- Group changes by: Added, Changed, Deprecated, Removed, Fixed, Security
- Write entries as you make changes, not at release time
- Link version headers to git diffs

## `#[non_exhaustive]`

Use `#[non_exhaustive]` on public enums and structs to preserve semver flexibility:

```rust
#[non_exhaustive]
pub enum Error {
    NotFound,
    PermissionDenied,
    // Adding a variant later is NOT a breaking change
}

#[non_exhaustive]
pub struct Config {
    pub name: String,
    pub timeout: u64,
    // Adding a field later is NOT a breaking change
}
```

- Consumers must use `..` in struct patterns and `_` in match arms
- Only use on types you expect to extend — it adds friction for consumers
- Don't use on internal types — it's only meaningful for public APIs

## License Consistency

The project license must be declared consistently across all indicators. When changing the license, update every location in a single commit.

### Required Locations

| Location                   | Field / Content                         | Example                                           |
| -------------------------- | --------------------------------------- | ------------------------------------------------- |
| `Cargo.toml`               | `[workspace.package] license`           | `license = "MIT"`                                 |
| Member `Cargo.toml`        | `license.workspace = true`              | Inherits from workspace                           |
| `LICENSE` or `LICENSE-MIT` | Full license text at project root       | MIT text with correct copyright holder and year   |
| `deny.toml`                | `[licenses] allow` list                 | Must include the project's own license identifier |
| `book.toml`                | (optional) footer or description        | Should not contradict                             |
| `README.md`                | License section or badge                | Link to `LICENSE` file                            |
| Source file headers        | (if used) `// SPDX-License-Identifier:` | `// SPDX-License-Identifier: MIT`                 |

### SPDX Identifiers

Use [SPDX license identifiers](https://spdx.org/licenses/) in `Cargo.toml`. Common choices:

| Identifier          | License                                 |
| ------------------- | --------------------------------------- |
| `MIT`               | MIT License                             |
| `Apache-2.0`        | Apache License 2.0                      |
| `MIT OR Apache-2.0` | Dual-licensed (Rust ecosystem standard) |
| `BSD-2-Clause`      | Simplified BSD                          |
| `BSD-3-Clause`      | New BSD                                 |
| `GPL-3.0-only`      | GNU GPL v3                              |
| `LGPL-3.0-or-later` | GNU LGPL v3+                            |
| `MPL-2.0`           | Mozilla Public License 2.0              |

For dual licensing, use the SPDX expression syntax: `license = "MIT OR Apache-2.0"`. Provide both license files (`LICENSE-MIT`, `LICENSE-APACHE`).

### Verification

Run `cargo deny check licenses` to verify all dependencies comply with the allowed license list. The project's own license must appear in the `allow` list in `deny.toml`:

```toml
[licenses]
unlicensed = "deny"
allow = [
  "MIT",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Unicode-3.0",
]
```

### Copyright Year

- Use the year of first publication: `Copyright 2025`
- A range (`2025-2026`) is optional — the initial year establishes the copyright
- The copyright holder must match `[workspace.package] authors`

### Consistency Checklist

When changing or adding a license:

- [ ] `Cargo.toml` `license` field uses correct SPDX identifier
- [ ] `LICENSE` (or `LICENSE-MIT`, `LICENSE-APACHE`) file exists at project root with full text
- [ ] Copyright holder and year are correct in the license file
- [ ] `deny.toml` `[licenses] allow` includes the project's license
- [ ] `README.md` references the correct license
- [ ] `docs/LICENSE` (if present in mdbook) matches or links to the root license
- [ ] All member crates inherit with `license.workspace = true`
- [ ] Source file headers (if used) match the declared license
