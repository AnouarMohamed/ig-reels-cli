# 09 — Errors and Observability

## 1. Error taxonomy

Rust should expose typed error categories rather than string matching.

Top-level examples:

```text
ConfigError
GatewayError
ProtocolError
FeedError
DownloadError
CacheError
ProbeError
DecodeError
RenderError
AudioError
TerminalError
PlaybackError
ShutdownError
```

Use `thiserror` or equivalent pinned mechanism.

## 2. User-visible vs log detail

User status gets short safe message:

```text
download failed — skipping
ffmpeg unavailable
terminal too small — q quit
audio unavailable
```

Log receives structured context:
- operation id,
- Reel id,
- error category,
- child exit status,
- bounded redacted stderr tail.

Never full media URL/session/credentials.

## 3. Logging destination

Before TUI: diagnostics may use stderr.

During TUI: normal tracing goes to file under `IG_REELS_LOG_DIR`, not stdout/stderr.

Display errors through typed status commands.

## 4. Required structured fields

Use stable names where meaningful:

```text
component
operation_id
playback_generation
reel_id
phase
error_kind
child_program
child_exit_code
bytes_downloaded
frame_index
render_cols
render_rows
fps
```

Do not log every frame at info level. Per-frame logs belong trace-only and are disabled by default.

## 5. FFmpeg/ffprobe stderr

Drain continuously. Retain at most last 64 KiB for error context.

No unbounded String accumulation.

## 6. Metrics without telemetry

No external telemetry service in v1.0.

For profiling/debug logs, locally record bounded summary counters:
- frames decoded,
- frames presented,
- frames dropped,
- average render encode duration,
- max render encode duration,
- bytes written to display,
- downloads completed/failed.

Counters can be emitted at Reel end or debug level.

## 7. Error policy matrix

| Error | Policy |
|---|---|
| gateway unavailable at startup | fatal |
| auth/challenge required | fatal startup with clear manual action |
| feed one-call rate limit/unavailable | recoverable with bounded policy; no tight loop |
| one media 4xx/5xx | skip Reel |
| oversized download | skip Reel |
| probe malformed | skip Reel |
| FFmpeg missing | fatal preflight |
| ffprobe missing | fatal preflight |
| one Reel decode failure | skip Reel |
| audio device missing | session video-only degradation |
| one Reel audio decode fail | Reel video-only degradation |
| terminal below minimum at startup | fatal/clear message before raw mode |
| terminal becomes tiny during playback | suspend video drawing, keep quit/input |
| display stdout error | fatal shutdown |
| cache cleanup failure after restored terminal | warn/log; nonzero exit if appropriate |
