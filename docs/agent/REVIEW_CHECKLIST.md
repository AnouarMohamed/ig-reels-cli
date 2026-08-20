# Per-Task Review Checklist

## Scope
- [ ] Only task-allowed paths changed.
- [ ] No later-task implementation slipped in.
- [ ] Diff is small enough to review as one concept.

## Product identity
- [ ] No image protocol/GUI fallback added.
- [ ] mpv remains smoke-only.
- [ ] renderer mapping remains `▀` top-FG/bottom-BG where relevant.

## Architecture
- [ ] Python/Rust ownership boundary preserved.
- [ ] App state not moved into global shared mutex.
- [ ] stdout single-owner rule preserved.
- [ ] stale async result has id/generation handling where relevant.

## Safety
- [ ] No secrets/full media URLs logged.
- [ ] No shell command string.
- [ ] No unsafe path deletion.
- [ ] Remote text sanitized before terminal.
- [ ] No boundary `unwrap`/`expect`.

## Media
- [ ] Frame dimensions checked and sample height even.
- [ ] RGB frame length uses checked arithmetic.
- [ ] child stderr cannot deadlock pipe.
- [ ] cancellation owns/terminates child.

## Renderer
- [ ] no direct stdout in renderer.
- [ ] no per-cell heap allocation hidden in loop if avoidable.
- [ ] explicit cursor movement rather than newline/wrap.
- [ ] full frame buffered before display write.
- [ ] reset before status text.

## Tests
- [ ] Unit/integration tests added for new behavior.
- [ ] Task-specific gate actually run.
- [ ] Manual gate marked NOT RUN if not run.
- [ ] `git diff --check` clean.
- [ ] formatter/check/tests/clippy applicable commands pass.
