# 18 — Internal Event and Display Contracts

This file constrains cross-module messages so implementation does not invent incompatible ownership patterns.

## 1. Identity types

Use distinct semantic types/newtypes where practical:

```text
OperationId(u64)
PlaybackGeneration(u64)
FrameIndex(u64)
```

Counters are monotonically increasing within one process. Wrapping is not expected in realistic runtime; use checked/saturating policy or documented wrap handling rather than panicking.

## 2. AppEvent direction

Workers -> App only.

Representative contract:

```text
GatewayReady
GatewayFailed(AppError)
FeedLoaded { op, items }
FeedFailed { op, error }
DownloadReady { op, reel_id, cached }
DownloadFailed { op, reel_id, error }
ProbeReady { op, reel_id, media_info }
ProbeFailed { op, reel_id, error }
PlaybackStarted { generation }
PlaybackEnded { generation }
PlaybackFailed { generation, error }
AudioDegraded { generation, reason }
Input(InputAction)
TerminalResized(TerminalMetrics)
SignalInterrupt
WorkerFailed { component, error }
```

A worker never directly moves back/current/forward queues.

## 3. InputAction

Only:

```text
Next
Previous
TogglePause
Quit
```

Unknown keys are ignored before AppEvent emission or mapped to no-op. Resize is its own event.

## 4. Display architecture: two lanes

The display owner receives:

### A. Reliable control queue
A bounded mpsc-like queue for low-volume commands. Capacity target: 16.

Control commands must not be intentionally dropped:

```text
BeginGeneration { generation, geometry }
Status { generation: Option<Generation>, text }
SuspendTooSmall { cols, rows }
ClearTransientMessage
Shutdown
```

### B. Latest-frame slot
Video frames use **latest-value semantics**, not a normal accumulating queue.

Recommended implementation shape: Tokio watch-like channel containing:

```text
Option<Arc<RenderFrame>>
```

`Arc` here is immutable frame sharing and is allowed; the prohibition is against `Arc<Mutex<AppState>>`.

When producer publishes frames faster than terminal can display them, intermediate frame values are overwritten/collapsed. Display owner consumes the newest observed frame. This is desired.

## 5. DisplayOwner priority

Display loop must prioritize control over video frames.

Conceptual behavior:

```text
loop:
    if control ready:
        process control first
    else if latest frame changed:
        process newest frame
```

A biased `select` or equivalent is acceptable after verifying current Tokio API.

This guarantees new-generation/Shutdown commands are not starved by continuous frame traffic.

## 6. BeginGeneration semantics

On `BeginGeneration {G, geometry}`:
1. set `active_generation = G`,
2. store geometry,
3. reset attributes,
4. clear alternate-screen content/viewport once,
5. hide cursor if not already hidden,
6. render initial safe status if available,
7. ignore any frame whose generation != G.

After step 1, an old frame still present in latest-frame slot is stale and must be dropped.

## 7. Frame semantics

`RenderFrame` already contains cursor/color/glyph bytes for video region and reset at end.

On newest frame:
1. verify frame generation == active generation,
2. if terminal currently suspended-too-small, discard frame,
3. `write_all(frame.bytes)`,
4. render current status if status is coupled to frame or has changed according to implementation policy,
5. flush once per presented frame/control update as needed.

Do not partially write one frame from multiple tasks.

## 8. Status semantics

`Status { generation: Some(G) }` is discarded if G is not active.

`Status { generation: None }` is global (e.g. shutting down/terminal too small).

Status row is the **last terminal row** (`rows - 1`) in zero-based internal coordinates. Video available rows exclude this row.

Display owner explicitly moves cursor to status row and clears that row before writing sanitized/truncated status so remnants from longer previous text do not remain.

## 9. Resize semantics

Terminal resize event goes to App and updated metrics may also be made available to DisplayOwner through control.

Current `RenderGeometry` stays frozen.

If current geometry no longer fits visible area:
- DisplayOwner may clip safe writes if trivial,
- or enter `SuspendTooSmall` if below minimum,
- never allow terminal auto-wrap/scroll.

Next generation uses new geometry calculated by App/playback setup.

## 10. Shutdown semantics

`Shutdown` is reliable control.

DisplayOwner:
1. stops accepting/presenting frames,
2. resets attributes,
3. flushes,
4. exits its loop,
5. TerminalGuard restoration happens in owning shutdown path after display task is joined/ended.

DisplayOwner itself does not delete cache or stop gateway.

## 11. Boundedness invariant

- AppEvent queue bounded.
- Display control queue bounded.
- Video frame output is latest-value, not accumulating.
- Decoder RGB channel capacity 3.

No channel carrying frames/ANSI payloads is unbounded.
