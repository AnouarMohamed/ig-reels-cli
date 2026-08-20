# AI Coding Agent Rules — Mandatory

You are not the architect. The specification already made architecture decisions.

## 1. Before every task

You MUST output this checklist before editing:

```text
TASK: T-XXX <title>
READ:
- <files actually read>
EDIT PLAN:
- <exact path 1>: <one-sentence change>
- <exact path 2>: <one-sentence change>
NOT EDITING:
- <important adjacent files>
UNCERTAINTIES:
- none | <exact API/fact requiring verification>
```

If an edit path is not allowed by the task card, STOP.

## 2. One task only

Do not:
- implement later task functionality,
- create “future-proof” abstractions not required now,
- refactor neighboring modules,
- change dependency versions unless task allows it,
- change architecture because another approach feels easier.

## 3. No invented APIs

When a dependency call is uncertain:
1. inspect the pinned crate/package docs/source,
2. or write minimal code and let compiler verify,
3. cite/record what was verified in task report.

Never fabricate a method name/signature.

## 4. Boundary error rule

No runtime `unwrap`/`expect` for:
- env/config,
- socket,
- network,
- HTTP body,
- filesystem,
- process spawn/wait,
- FFmpeg/ffprobe output,
- terminal,
- audio.

Tests may use unwrap when failure should fail the test immediately.

## 5. Secrets

Never print/log:
- username/password,
- session JSON/settings,
- cookies,
- complete signed media URLs,
- whole environment.

Use Reel id and redacted URL state instead.

## 6. External processes

Never use:
- `sh -c`,
- `bash -c`,
- command strings assembled from data.

Use executable + argument vector. Explicitly set stdin/stdout/stderr ownership.

## 7. Terminal video product rule

The final renderer is text-cell-only.

If you are tempted to add:
- Kitty graphics,
- Sixel,
- iTerm images,
- terminal image crate,
- SDL/OpenGL/browser/window,
- mpv as final player,
STOP. That violates the product.

## 8. stdout rule

Once TUI mode begins, only `DisplayOwner` writes stdout.

Do not add `println!`, `eprintln!`, direct crossterm execution, or FFmpeg inherited stdout in workers.

## 9. State rule

Do not put App state behind `Arc<Mutex<_>>`.

Workers emit typed events. App loop applies transitions.

Every potentially stale async completion uses operation id/generation.

## 10. Renderer rule

v1.0 video cell mapping is fixed:

```text
top RGB -> foreground
bottom RGB -> background
glyph -> U+2580 '▀'
```

Do not substitute ASCII ramps, Braille, quadrant glyphs, or image protocols unless a future task explicitly changes renderer mode.

## 11. Geometry rule

Do not assume terminal cells are square.

Use `spec/04_MEDIA_PROBE_AND_GEOMETRY.md`. Do not improvise new aspect formulas.

## 12. Test rule

A task is not done because code “looks right.” Run:
- standard commands relevant to touched language,
- task-specific tests/gates,
- exactly report failures.

Never claim a manual test was performed when it was not.

Use status:
- `PASS`
- `FAIL`
- `NOT RUN`
- `NOT VERIFIED`

## 13. If a task fails

Do not expand scope. Report:

```text
BLOCKED
Task: T-XXX
Command/test:
Observed error:
Likely boundary:
Spec/decision affected:
Smallest next diagnostic:
```

If a locked decision must change, STOP and propose ADR.

## 14. End-of-task report

Every task completion ends with:

```text
TASK RESULT: PASS | BLOCKED
CHANGED FILES:
- ...
TESTS/GATES:
- PASS ...
- NOT RUN ...
VERIFIED DEPENDENCY FACTS:
- ...
DIFF STAT:
<git diff --stat>
OUT-OF-SCOPE WORK DONE:
- none
REMAINING TASK-LOCAL LIMITATION:
- none | ...
NEXT TASK:
- T-YYY only if current task PASS
```
