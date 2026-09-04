# Contributing to dig-events-protocol

Thanks for your interest in improving the DIG event protocol. This crate owns the canonical
blockchain→app event contract that the DIG node/wallet engine emits and apps subscribe to.
Please read this before opening a PR.

## Reporting an issue

- Use this repo's [GitHub issues](https://github.com/DIG-Network/dig-events-protocol/issues).
- A good issue states the problem clearly, references the file/line if it's a code issue, and describes
  what you expected vs. what happened. Actionable details beat vague descriptions.
- For security concerns, read `SECURITY.md` first and **report vulnerabilities privately** rather than
  opening a public issue.

## Prerequisites

- [Rust](https://rustup.rs), stable (via `rustup`). The crate specifies `rust-version = "1.75.0"` in
  `Cargo.toml`; `rustup` will use stable unless a `rust-toolchain.toml` is present (it is not).
- No special build-time prerequisites — this is a pure leaf crate (no `build.rs`, no vendored code, no
  proc-macro generation).

## Build & test

```sh
# build the crate
cargo build

# run the full test suite
cargo test
```

This is a pure data/contract crate (no I/O, no async runtime), so every path executes deterministically
in tests. The conformance KATs (`tests/conformance.rs`) freeze the wire format against drift — keep them
passing.

## The gate (must pass before a PR is merged)

CI runs these on every PR (`.github/workflows/ci.yml`); run them locally first to stay green:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo doc --no-deps
cargo llvm-cov nextest --all --retries 2 --fail-under-lines 80 --lcov --output-path lcov.info
```

All six checks must pass:
1. **Format** — `cargo fmt --all --check`; auto-fix with `cargo fmt --all`.
2. **Clippy** — `cargo clippy --all-targets -- -D warnings`; warnings are errors.
3. **Build (release)** — `cargo build --release` must produce a binary.
4. **Doc** — `cargo doc --no-deps` must generate docs without errors.
5. **Tests** — `cargo test` runs the test suite.
6. **Coverage** — `cargo llvm-cov nextest --all --retries 2 --fail-under-lines 80` requires ≥80% line
   coverage (ecosystem standard). Install [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
   and [cargo-nextest](https://github.com/nextest-rs/nextest) if not already present; `nextest`
   instruments the SAME execution as `llvm-cov` so coverage is accurate.

The gate also includes:
- **Commitlint** (`.github/workflows/commitlint.yml`) — commit messages follow Conventional Commits
  (`type(scope): summary`; see below).
- **Version increment** (`.github/workflows/ensure-version-increment.yml`) — every PR must bump the
  version in `Cargo.toml` using SemVer (see below).

## Commit conventions

- Use [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): summary`, where
  `type` ∈ `feat|fix|docs|style|refactor|perf|test|build|ci|chore`. Keep one logical change per commit
  where practical.
- End every commit Claude helps author with a `Co-Authored-By: Claude <noreply@anthropic.com>` trailer.
- The commit message drives the SemVer bump (§2.4 of the ecosystem contract):
  - `fix(…)` → patch version (e.g., `0.1.2` → `0.1.3`)
  - `feat(…)` → minor version (e.g., `0.1.2` → `0.2.0`)
  - `BREAKING CHANGE:` in the footer or `type!` → major version (e.g., `0.1.2` → `1.0.0`)

## Pull requests

1. Branch from `main`.
2. Make the gate green locally (see "The gate" above).
3. **Bump the version** in `Cargo.toml` — `[package].version` — using SemVer before opening the PR.
   Commitlint + the version-increment gate both check it.
4. Open a PR with a clear description of the change and its rationale; reference any related issue.
   Keep the diff focused. The PR title should follow Conventional Commits format.
5. Rebase `main` in before merge (branch protection requires `strict = true` — your branch must be up to
   date). All checks must be green before merge.

## Spec & conformance

The normative contract lives in `SPEC.md`. The wire format is frozen by golden-JSON KATs
(`tests/conformance.rs`): every event variant, the envelope, and the kind list round-trip against
byte-stable fixtures. A change that alters the wire shape must update both `SPEC.md` and the golden
fixtures to stay green.
