# 10 — Test Strategy

## 1. Testing principle

Unstable external systems appear only in explicit manual smoke/soak gates. CI proves everything else with fake/local deterministic boundaries.

## 2. Test layers

### Pure unit tests
- IPC serialization/framing helpers,
- feed/dedup/history,
- safe URL Debug,
- sanitizer,
- geometry math,
- RGB indexing,
- half-block mapping,
- ANSI state coalescing,
- App transitions,
- cache path validation.

### Local integration
- fake UDS gateway,
- local HTTP server,
- generated MP4/AAC fixture,
- ffprobe parse,
- FFmpeg raw frame pipe,
- cancellation/child cleanup,
- terminal renderer golden bytes without real terminal.

### Manual visual
- generated clip rendered in actual terminals,
- real Reel visual playback,
- resize behavior,
- terminal restoration.

### Manual Instagram
- fresh/session login path,
- feed batch,
- real URL download,
- real audio,
- 30-Reel soak.

## 3. Generated AV fixture

Create deterministic test clip in `tests/fixtures/generated/` or generate on demand; do not commit large binary unless justified.

Fixture should contain for >=20 s:
- moving `testsrc`/`testsrc2` style color pattern,
- strong RGB regions,
- diagonals/edges,
- motion across full frame,
- audio sine/tone.

A second short portrait fixture (e.g. 360×640 or equivalent generated source) is mandatory for geometry tests.

Script records exact ffmpeg command and verifies output with ffprobe.

## 4. Geometry golden table

Pure tests include at least:

| source DAR | terminal | cell ratio | expected property |
|---|---|---:|---|
| 9:16 | 120×40 | 2.0 | exactly 44×78 samples |
| 16:9 | 120×40 | 2.0 | width-limited, sample H even |
| 1:1 | 80×24 | 2.0 | square physical display |
| 9:16 | 80×24 | 1.8 | corrected ratio within 3% |
| 16:9 | 40×10 | 2.4 | fits and even height |

Freeze exact integer outputs after deterministic rounding function is implemented and reviewed.

## 5. Renderer golden tests

Keep tiny frames as literal byte arrays; no image files needed.

Test semantic output:
- top/bottom colors,
- glyph count = W * H/2,
- explicit cursor row placement,
- color reset,
- no raw newline row movement,
- no forbidden image protocol markers.

Do not snapshot megabytes of ANSI for large frames.

## 6. Local HTTP cases

Server endpoints:
- 200 fixed body,
- chunked/no Content-Length,
- declared oversized length,
- body grows over limit,
- 302 -> HTTPS policy simulation where feasible,
- too many redirects,
- redirect to HTTP rejected,
- delayed response timeout,
- mid-stream disconnect.

If local TLS test setup is too heavy, separate pure redirect-policy tests from HTTP body integration, but HTTPS scheme rules still require coverage.

## 7. Process cancellation tests

Start FFmpeg on long generated source. Cancel quickly. Assert:
- task returns within bounded time,
- child is no longer running,
- stdout reader exits,
- stderr drainer exits,
- no stale frame accepted after generation change.

## 8. Terminal restoration fault tests

Where possible use a pseudo-terminal/manual harness.

At minimum manually verify:
- q,
- Ctrl-C,
- induced playback error,
- induced panic in development harness,
all restore cursor/raw/alternate-screen/wrap state.

## 9. Performance test

Use local generated 120-column-equivalent frame workload.

Measure separately:
- RGB->ANSI encode time,
- bytes/frame,
- stdout write time in real terminal visual benchmark,
- frames presented/dropped.

The unit benchmark must not claim terminal throughput from `/dev/null` alone.

## 10. Soak

Live 30-Reel session, including:
- at least 10 Next actions,
- 5 Previous/Next retraces,
- 5 pause/resumes,
- several resizes,
- at least one rapid navigation sequence,
- natural ends if practical.

After quit inspect:
- no ffmpeg/mpv child,
- no current run cache,
- bounded logs,
- process memory did not monotonically explode,
- terminal normal.
