# 07 — Application State Machine

## 1. Why explicit state exists

Async workers must not become the state machine. `App` is the only authority deciding what Reel is current and which operation is valid.

## 2. Phases

```text
Starting
ConnectingGateway
Preflight
LoadingFeed
Buffering
Playing
Paused
Switching
RecoverableError
FatalError
ShuttingDown
Exited
```

Equivalent typed substates are allowed if transitions/invariants remain testable.

## 3. Core App data

Conceptually:

```text
phase
current: Option<CachedReel>
back_history
forward_history
unseen_metadata
unseen_ready
seen_ids
active_feed_op: Option<OpId>
active_download_op: Option<OpId>
active_probe_op: Option<OpId>
active_playback_generation: Option<Generation>
latest_terminal_metrics
audio_capability
last_recoverable_error
shutdown_reason
```

## 4. Event vocabulary

Minimum:

```text
GatewayReady
GatewayFailed(error)
PreflightReady(info)
PreflightFailed(error)
FeedLoaded(op_id, items)
FeedFailed(op_id, error)
DownloadReady(op_id, cached)
DownloadFailed(op_id, reel_id, error)
ProbeReady(op_id, reel_id, media_info)
ProbeFailed(op_id, reel_id, error)
PlaybackStarted(generation)
PlaybackEnded(generation)
PlaybackFailed(generation, error)
AudioDegraded(generation, reason)
KeyNext
KeyPrevious
KeyPause
KeyQuit
TerminalResized(metrics)
SignalInterrupt
WorkerPanicked(name)
```

## 5. Stale-result rule

For every event containing op id/generation:
- compare with current active id,
- if mismatch: ignore state mutation,
- optionally log stale result,
- never turn stale failure into current UI error.

## 6. Startup

```text
Starting
 -> ConnectingGateway
 -> Preflight
 -> LoadingFeed
 -> Buffering(first)
 -> Playing
```

Prefer entering raw/alternate screen only after gateway/feed/preflight has succeeded enough to avoid showing login errors inside broken TUI mode.

## 7. Playing

```text
KeyPause       -> Paused
KeyNext        -> Switching(next)
KeyPrevious    -> Switching(previous) if possible
PlaybackEnded  -> Switching(next)
PlaybackFailed -> recover/skip policy
KeyQuit        -> ShuttingDown
SIGINT         -> ShuttingDown
Resize         -> update metrics, current geometry frozen
```

## 8. Paused

```text
KeyPause -> Playing
KeyNext -> Switching(next)
KeyPrevious -> Switching(previous)
KeyQuit/SIGINT -> ShuttingDown
Resize -> update metrics
```

## 9. Switching

Switching is not reentrant.

While an old generation is being cancelled and new current prepared:
- repeated navigation events are ignored/coalesced according to one deterministic policy,
- v1.0 policy: ignore all Next/Previous until phase leaves Switching,
- Quit/SIGINT is always honored immediately.

Do not launch two playback generations concurrently because user held a key.

## 10. Previous semantics

If back empty -> no-op.

Otherwise navigation model from `spec/03` applies. `unseen_ready` remains separate.

## 11. Next semantics

Priority:
1. forward history,
2. unseen_ready,
3. buffer/download unseen metadata,
4. request feed refill if necessary.

## 12. Failure classification

Recoverable examples:
- one Reel HTTP failure,
- one malformed media file,
- one Reel audio decode failure,
- FFmpeg failure on one Reel,
- empty/duplicate-only batch if refill can continue.

Fatal examples:
- incompatible protocol,
- terminal preflight permanently unusable,
- required ffmpeg/ffprobe missing,
- cache root unsafe/unusable,
- gateway cannot be reached after startup policy,
- terminal guard cannot be established for player.

## 13. Shutdown

On first shutdown trigger:
1. phase -> ShuttingDown,
2. reject new work,
3. mark playback inactive,
4. clear/suspend display generation,
5. cancel global token,
6. cancel/kill playback child,
7. stop pending downloads/feed tasks,
8. wait boundedly for tracked workers,
9. restore terminal,
10. remove current run cache,
11. flush logs,
12. phase -> Exited.

Second Ctrl-C during a hung graceful shutdown may force a narrower emergency path, but must still attempt terminal restoration before process exit.

## 14. State-machine unit tests

Pure tests must cover:
- startup happy path,
- previous at start no-op,
- previous then next retraces forward,
- new unseen next clears forward branch,
- combined history bound,
- natural end equivalent to next,
- pause toggles,
- stale feed result ignored,
- stale download result ignored,
- stale playback ended/failed ignored,
- resize does not change current generation geometry,
- quit accepted from every non-exited state,
- repeated navigation during Switching does not create second transition.
