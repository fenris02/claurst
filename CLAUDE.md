# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Rust bin-packing project. Always use edition 2024, MSRV 1.91.0, resolver 3.

## Build Commands

- **Build:** `cargo build`
- **Run:** `cargo run`
- **Test:** `cargo nextest run` (preferred) or `cargo test` (fallback for doc-tests)
- **Run single test:** `cargo nextest run test_name`
- **Lint:** `cargo +nightly clippy --all-targets --all-features -- -D warnings`
- **Format Rust:** `cargo +nightly fmt`
- **Format TOML:** `taplo format`
- **Format Markdown:** `dprint fmt`
- **Check:** `cargo check` (fast compile check without producing binary)
- **Supply chain audit:** `cargo deny check` (advisories, licenses, bans)
- **Safety check:** `cargo careful test` (stdlib debug assertions + UB checks)
- **Build docs:** `mdbook build` (project book) or `cargo doc --no-deps` (API docs)

## Structure

- `Cargo.toml` — workspace root (edition 2024, resolver 3, workspace lints)
- `.rustfmt.toml` — rustfmt configuration (Edition 2024 style, nightly features)
- `.clippy.toml` — clippy configuration (disallowed methods/types/macros, MSRV)
- `rust-toolchain.toml` — pins toolchain channel and components
- `book.toml` — mdbook configuration for project documentation
- `docs/` — mdbook source (symlinks to `.claude/docs/` for standards)
- `bin/src/main.rs` — binary entry point
- `lib/src/lib.rs` — library crate

## Rust Standards

Detailed guidelines are organized into include files under `.claude/docs/`:

@.claude/docs/type-system-and-api-design.md
@.claude/docs/ownership-and-memory.md
@.claude/docs/error-handling.md
@.claude/docs/concurrency-and-async.md
@.claude/docs/style-and-conventions.md
@.claude/docs/lints-and-tooling.md
@.claude/docs/testing.md
@.claude/docs/dependencies-and-supply-chain.md
@.claude/docs/tracing-and-observability.md
@.claude/docs/serde-patterns.md
@.claude/docs/feature-flags.md
@.claude/docs/workspace-and-crate-structure.md
@.claude/docs/build-scripts-and-proc-macros.md
@.claude/docs/ci-pipeline.md
@.claude/docs/versioning-and-release.md
@.claude/docs/unsafe-and-ffi.md
@.claude/docs/unsafe-rules-reference.md
@.claude/docs/unsafe-checklists.md
@.claude/docs/unsafe-examples.md
@.claude/docs/rust-1.85-1.93-apis.md
