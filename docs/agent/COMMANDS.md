# Standard Proof Commands

Run only commands applicable to the current task, plus card-specific gates.

## Repository

```bash
git status --short
git diff --check
git diff --stat
```

## Python

From `py-ig-gateway/`:

```bash
python -m pytest -q
python -m compileall .
```

If pytest is not yet a declared dependency in the scaffold, the task that introduces tests must explicitly add the dev dependency/mechanism before using it.

## Rust

From `rust-tui/`:

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

During early dependency bootstrap before Cargo.lock exists, use the task-specific command; after lock creation always use `--locked` for proof.

## Search for forbidden runtime panics in source

Use as review aid, not perfect parser:

```bash
rg -n 'unwrap\(|expect\(' rust-tui/src
```

Every match at an external boundary must be removed or justified as an internal invariant unreachable after validated construction.

## Search for accidental final renderer protocol scope creep

```bash
rg -ni 'kitty|sixel|iterm|inline image|graphics protocol' rust-tui py-ig-gateway scripts README.md docs || true
```

Allowed matches are documentation saying they are forbidden; code/dependency matches are release blockers.

## Secret/runtime artifact check

```bash
git ls-files | rg '(^|/)(\.env|session\.json|target/|venv/|tmp-cache|run-|logs?/)' && exit 1 || true
```

Adjust only if repository intentionally tracks a harmless fixture with a clearly different name.

## System tools

```bash
ffmpeg -version
ffprobe -version
mpv --version   # only smoke gate
```

## Process leak checks after manual playback

```bash
pgrep -af 'ffmpeg|mpv' || true
```

Interpret carefully: unrelated user processes may exist. Confirm PIDs belong to project before claiming leak.

## Release build

```bash
cd rust-tui
cargo build --release --locked
```
