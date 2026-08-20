# 11 — Build and Ship Plan

The implementation is divided into gates. **Do not skip a gate because later code seems more exciting.**

## Gate A — Safe repository/config
Goal: empty project cannot accidentally commit secrets/runtime artifacts.

Exit evidence:
- ignore rules verified,
- dependency pins/lock strategy established,
- no Instagram/media behavior.

## Gate B — IPC without Instagram
Goal: prove Python/Rust process boundary with fake data.

Exit evidence:
- Python frame codec tests,
- fake UDS gateway,
- Rust codec/client,
- bidirectional contract tests.

## Gate C — Real Instagram gateway
Goal: isolate the unstable upstream once.

Exit evidence:
- ReelDTO mapper tests,
- auth-free ping,
- lazy session reuse/login,
- live Reels batch manual smoke,
- no URL/secret leakage.

**Freeze gateway after this gate.** Later renderer tasks do not touch it.

## Gate D — Download/cache
Goal: obtain safe local MP4 independent of player.

Exit evidence:
- cache path tests,
- local HTTP tests,
- real Reel download,
- atomic `.part` behavior,
- one-ahead model unit tests.

## Gate E — External media proof
Goal: prove local media itself works before custom engine.

Exit evidence:
- generated fixture,
- mpv `tct` smoke on real downloaded Reel.

No Rust terminal navigation yet.

## Gate F — Probe and geometry
Goal: know exact media/terminal geometry.

Exit evidence:
- ffprobe JSON parse tests,
- cell-aspect detection/fallback,
- pure geometry golden table,
- portrait/landscape fixtures.

## Gate G — FFmpeg RGB decoder
Goal: produce bounded whole RGB24 frames.

Exit evidence:
- command builder,
- exact frame sizing,
- EOF/truncation handling,
- bounded stderr,
- cancellation test.

## Gate H — Text renderer
Goal: turn RGB into video-looking terminal text.

Exit evidence:
- half-block mapping tests,
- ANSI encoder tests,
- sanitizer/status integration,
- full-frame builder,
- local visual generated-clip smoke at 15 FPS,
- aspect/cursor/no-scroll checks.

At this point the project already has its portfolio-defining renderer.

## Gate I — Audio and sync
Goal: add audio without destabilizing renderer.

Exit evidence:
- generated audio spike,
- real Reel rodio spike,
- audio/session capability state,
- video-only clock,
- audio-master scheduler,
- pause/resume sync check.

## Gate J — Terminal/App integration
Goal: make it interactive and race-safe.

Exit evidence:
- TerminalGuard,
- input worker,
- DisplayOwner,
- pure App state tests,
- generation cancellation,
- local fixture app navigation.

## Gate K — Live end-to-end
Goal: use real feed with prefetch/navigation.

Exit evidence:
- live fetch -> download -> probe -> text render -> audio,
- Next/Previous/Pause/Quit,
- refill/prefetch,
- resize safety,
- recover from at least one forced media failure.

## Gate L — Hardening
Goal: prove no hidden lifecycle/cleanup bugs.

Exit evidence:
- 30-Reel soak,
- rapid nav stress,
- Ctrl-C,
- child leak check,
- cache cleanup check,
- log redaction scan,
- performance budget check.

## Gate M — Ship
Goal: clean-clone reproducible release.

Exit evidence:
- CI green without secrets,
- release-mode build,
- launcher/bootstrap docs tested,
- release archive clean,
- version/changelog/readme complete,
- final checklist signed off.

The exact atomic implementation order is in `agent/TASK_CARDS.md`.
