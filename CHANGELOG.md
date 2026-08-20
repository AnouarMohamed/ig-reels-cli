# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
- feat(T-005): implement framed MessagePack codec
  - create py-ig-gateway/protocol.py with encode_frame and decode_frame functions
  - add tests for zero/oversize/truncated/invalid payload cases
  - verified tests pass with sequential execution
- feat(T-004): create shared configuration/path parser skeleton
  - create rust-tui/src/config.rs with environment variable parsing and validation
  - update rust-tui/src/main.rs to use Config and print configuration
  - add tests for valid/default/invalid FPS, max cols, max rows, fallback aspect
  - verified tests pass with sequential execution
  - verified binary works with custom environment variables

## [0.1.0] - 2026-08-20
### Added
- Initial repository skeleton per T-001
  - added missing top-level README.md, LICENSE, CHANGELOG.md
  - created docs/guides/ and placeholder files
  - moved Python gateway files from py-auth-daemon/ to py-ig-gateway/
  - created missing config.py and protocol.py placeholders
  - updated .gitignore to ignore rust-tui/target/
  - removed duplicate top-level manifesto.md
  - ran security scan and code quality checks (ruff, cargo clippy, etc.)
- Python gateway dependencies pinned per T-002
  - created py-ig-gateway/requirements.txt with instagrapi==2.18.16, msgpack==1.2.1
  - verified clean venv install succeeds
- Rust crate baseline per T-003
  - created minimal Cargo.toml, Cargo.lock, src/main.rs
  - verified cargo check --locked passes
  - verified rustfmt check passes
- Terminal video specification v3
  - installed terminal video specification v3 docs
- Phase 1 - Python auth daemon
  - implemented Instagram client and UDS server
