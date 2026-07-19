# Changelog

All notable changes to this project are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and
[Conventional Commits](https://www.conventionalcommits.org).

## [0.1.2] - 2026-07-19

### Bug Fixes
- **events:** `Amount` now serializes ALWAYS as a decimal string (every value, small or large) so a
  JS/TS consumer reads it as one `bigint` — one code path, no precision loss past
  `Number.MAX_SAFE_INTEGER`. Deserialization accepts the canonical string and, leniently, a bare
  number. Aligns the extracted crate with the JS-safe-integer contract of its extraction source (#1112).

## [0.1.1] - 2026-07-19

### Documentation
- **dig-events-protocol:** Full protocol interface in README (#1)

## [0.1.0] - 2026-07-19

### Features
- Seed dig-events-protocol v0.1.0 — canonical blockchain→app event contract


