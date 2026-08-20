# Risk Register and Stop Conditions

The coding agent is not allowed to hide a failed assumption. Each risk has a mandatory proof point.

| Risk | Failure consequence | Proof point | Required response |
|---|---|---|---|
| instagrapi cannot provide usable Reels/Discover metadata | no product feed | live gateway smoke | stop; inspect upstream; no scraping fallback |
| returned media URL requires Instagram cookies/session in Rust | process boundary invalid | first real download | stop; ADR required; do not leak cookies over IPC |
| signed URL expires before queue/prefetch use | intermittent 4xx | 30-Reel soak | use skip/refill first; ADR only if systematic |
| rodio cannot decode actual Reel MP4/AAC | audio architecture invalid | audio spike + real Reel test | stop; ADR before FFmpeg audio extraction |
| `Player::get_pos()` is unsuitable as stable AV clock | drift/scheduling instability | AV fixture and real Reel test | stop; ADR before alternate clock |
| terminal truecolor output is unsupported | colors unusable | preflight/manual | fail with clear unsupported-terminal error; do not switch to image protocol |
| cell pixel dimensions unavailable/unreliable | wrong aspect | geometry tests/manual | fallback to ratio 2.0; allow future explicit override |
| ANSI throughput at 15 FPS/120 cols is unusable | stutter | benchmark + visual smoke | first optimize SGR/write path or lower explicit budget via ADR; no image protocol |
| full-frame repaint tears badly in target terminal | poor UX | visual smoke | profile/write batching; dirty-region renderer requires scoped ADR/task |
| FFmpeg stdout/stderr handling deadlocks | frozen playback | decoder cancellation/soak | fix before integration |
| stale playback writes after navigation | corrupted display | rapid navigation test | fix generation/output ownership before release |
| terminal restoration fails after panic/Ctrl-C | user terminal damaged | fault-injection tests | release blocker |
| gateway/session flow triggers repeated fresh login | account risk | restart manual test | fix protected gateway code only |
| remote caption injects ESC/control bytes | terminal injection | sanitizer tests | release blocker |
| cache cleanup can escape app-owned run directory | destructive deletion | path safety tests | release blocker |

## Mandatory stop conditions

Stop the current task and report evidence if any of these occurs:

- a locked decision must change;
- real media download requires auth headers/cookies unavailable by design;
- current instagrapi no longer has a viable read-only Reels/Discover route;
- real Reel audio fails the pinned rodio architecture;
- a dependency API materially differs from the documented baseline;
- a task requires editing a file outside its allowed paths;
- a test can pass only by weakening security, redaction, cleanup, or terminal ownership;
- rendering quality is proposed to be “fixed” with Kitty/Sixel/iTerm images.

The correct next output is: evidence, failing command/test, affected decision id, and an ADR proposal. It is not an improvised patch.
