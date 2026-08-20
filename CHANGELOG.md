# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
- fix(T-001): align repository skeleton with spec
  - add missing top-level README.md, LICENSE, CHANGELOG.md
  - create docs/guides/ and placeholder files
  - move Python gateway files from py-auth-daemon/ to py-ig-gateway/
  - create missing config.py and protocol.py placeholders
  - update .gitignore to ignore rust-tui/target/
  - remove duplicate top-level manifesto.md
  - run security scan and code quality checks (ruff, cargo clippy, etc.)
- feat(T-003): create rust crate and locked baseline
  - create minimal Cargo.toml, Cargo.lock, src/main.rs
  - verify cargo check --locked passes
  - verify rustfmt check passes
- feat(T-002): pin python gateway dependencies
  - create py-ig-gateway/requirements.txt with instagrapi==2.18.16, msgpack==1.2.1
  - verify clean venv install succeeds
- docs: install terminal video specification v3
- chore: add .gitignore and empty .env file
- feat: implement Phase 1 - Python auth daemon (IG client and UDS server)

## [0.1.0] - 2026-08-20
### Added
- Initial repository skeleton per T-001
- Python gateway dependencies pinned per T-002
- Rust crate baseline per T-003
