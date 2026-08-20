# How to Build This Project With a Weak Coding Agent

The project is deliberately documented so model intelligence is not the primary safety mechanism.

## 1. Never give the model the whole mission

Bad prompt:

```text
Build IG-Reels-CLI from these docs.
```

Good workflow:

```text
AGENT_RULES.md
COMMANDS.md
one task card
only specs/ADRs listed by that task
existing allowed source files
latest compiler/test output
```

## 2. Use one chat/thread per small task when possible

Long agent contexts cause it to reuse outdated assumptions. A fresh task context plus current code is safer.

## 3. The exact cycle

For T-XXX:
1. ensure clean git status,
2. paste `WEAK_MODEL_PROMPT.md`,
3. paste T-XXX only,
4. provide required specs,
5. require edit-plan block,
6. reject any unallowed path before code is changed,
7. let it implement,
8. run tests immediately,
9. feed exact error only,
10. repeat until PASS or BLOCKED,
11. run `REVIEW_CHECKLIST.md`,
12. commit `T-XXX <short title>`,
13. move to next card.

## 4. Never accept “should work”

Every task report must distinguish:
- PASS,
- FAIL,
- NOT RUN,
- NOT VERIFIED.

A manual gate cannot be converted to PASS by reasoning about code.

## 5. Protect the gateway

After T-016 passes, ordinary Rust/media tasks never edit Python Instagram code. If live playback fails, first isolate whether failure is:
- IPC,
- URL download,
- local media,
- probe,
- decode,
- renderer,
- audio,
- state.

Do not poke login code because “Instagram is involved.”

## 6. Protect the renderer identity

If the model suggests Kitty/Sixel because ANSI is slow or low-resolution, reject it. The project exists specifically to render video as text cells.

If performance is bad, use `spec/16_PERFORMANCE_BUDGET.md` optimization order.

## 7. Use deterministic inputs before live inputs

Order:
- fake gateway before Instagram,
- local HTTP before CDN,
- generated MP4 before Reel,
- literal RGB bytes before FFmpeg,
- fake clock before rodio clock,
- local fixture playlist before live feed.

This is the main reason a weak agent can succeed.

## 8. When an ADR is required

Do not let the model write an ADR just to avoid a difficult bug. ADR is required only when evidence disproves a locked architecture assumption.

ADR proposal must state:
- current decision id,
- observed evidence,
- commands/tests reproducing it,
- alternatives considered,
- smallest replacement decision,
- migration impact.

## 9. Commit discipline

One passing task -> one commit.

If T-043 fails after T-040/T-041/T-042 are green, you can reset/rework T-043 without destroying proven foundations.

## 10. Ship discipline

T-082 is not ceremonial. Every mandatory acceptance item needs evidence. `NOT RUN` means no release.
