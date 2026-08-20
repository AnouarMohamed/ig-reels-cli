# 13 — v1.0 Acceptance Criteria

A check marked **MANDATORY** blocks release.

## A. Repository/security

- [ ] MANDATORY `.env` ignored.
- [ ] MANDATORY session files ignored and outside repo by default.
- [ ] MANDATORY Cargo.lock committed.
- [ ] MANDATORY Python runtime deps exactly pinned.
- [ ] MANDATORY release archive secret scan clean.
- [ ] MANDATORY full signed URLs absent from normal logs.

## B. IPC/gateway

- [ ] MANDATORY Python/Rust framing contract tests green.
- [ ] MANDATORY fake gateway ping/get_reels integration green.
- [ ] MANDATORY gateway ping works with no Instagram credentials/network auth.
- [ ] MANDATORY live gateway returns normalized Reel batch.
- [ ] MANDATORY session reuse works on ordinary restart without unnecessary fresh login.
- [ ] MANDATORY challenge/auth failure surfaces cleanly; no automatic challenge bypass.

## C. Download/cache

- [ ] MANDATORY redirects capped and HTTPS-only.
- [ ] MANDATORY file size cap enforced with/without Content-Length.
- [ ] MANDATORY failed/cancelled download never becomes final cache hit.
- [ ] MANDATORY current run cache path is private/owned.
- [ ] MANDATORY cleanup cannot escape run directory.
- [ ] MANDATORY real Reel MP4 downloads successfully without sending Instagram credentials from Rust.

## D. Media probe/geometry

- [ ] MANDATORY ffmpeg and ffprobe preflight errors are clear.
- [ ] MANDATORY portrait/landscape/square geometry tests pass.
- [ ] MANDATORY sample height always even.
- [ ] MANDATORY predicted display DAR within 3% for standard-size golden cases.
- [ ] MANDATORY unavailable terminal pixel metrics fall back to 2.0 ratio safely.
- [ ] MANDATORY orientation behavior agrees between probe and decoder on tested rotated fixture or rotated handling is explicitly proved irrelevant for target media.

## E. Text-video renderer

- [ ] MANDATORY output video path uses only UTF-8 text + terminal control sequences.
- [ ] MANDATORY no Kitty/Sixel/iTerm image code/dependency/protocol.
- [ ] MANDATORY one cell maps top sample to FG, bottom to BG, glyph `▀`.
- [ ] MANDATORY RGB indexing/golden tests pass.
- [ ] MANDATORY renderer rejects malformed frame sizes.
- [ ] MANDATORY full frame constructed in memory before display write.
- [ ] MANDATORY DisplayOwner is only TUI stdout writer.
- [ ] MANDATORY generated portrait test clip is visually recognizable in text mode.
- [ ] MANDATORY no scrolling/wrap artifacts in visual test.
- [ ] MANDATORY no stale cells between differently sized Reel generations.
- [ ] MANDATORY remote ESC/control injection test cannot alter terminal.

## F. Playback/audio

- [ ] MANDATORY FFmpeg decoder outputs whole bounded RGB frames.
- [ ] MANDATORY FFmpeg cancellation leaves no child.
- [ ] MANDATORY generated MP4/AAC rodio spike passes on acceptance machine.
- [ ] MANDATORY at least one real Reel audio test passes, or an approved replacement ADR exists before release.
- [ ] MANDATORY video-only fallback works when audio unavailable.
- [ ] MANDATORY 20s AV fixture shows no accumulating drift.
- [ ] MANDATORY 3s pause/resume does not create permanent offset.

## G. Interaction/state

- [ ] MANDATORY Next keys work.
- [ ] MANDATORY Previous keys work.
- [ ] MANDATORY backward then Next retraces forward history first.
- [ ] MANDATORY Space toggles pause.
- [ ] MANDATORY natural end advances.
- [ ] MANDATORY repeated navigation during Switching cannot start overlapping generations.
- [ ] MANDATORY stale operation/generation events ignored.
- [ ] MANDATORY one-ahead prefetch works during normal playback.

## H. Terminal lifecycle

- [ ] MANDATORY q restores raw/alternate/cursor/wrap state.
- [ ] MANDATORY Ctrl-C restores terminal.
- [ ] MANDATORY induced playback error restores terminal on fatal exit.
- [ ] MANDATORY terminal-too-small state keeps Quit usable.
- [ ] MANDATORY current Reel survives resize without process corruption; next Reel adopts new geometry.

## I. Performance/soak

- [ ] MANDATORY 15 FPS/120-column generated visual benchmark is usable on primary acceptance terminal.
- [ ] MANDATORY renderer does not allocate unboundedly per Reel.
- [ ] MANDATORY decoded channel capacity is bounded.
- [ ] MANDATORY 30-Reel soak completes.
- [ ] MANDATORY after soak: no ffmpeg/mpv children.
- [ ] MANDATORY after quit: current run media cache removed.
- [ ] MANDATORY memory/disk use remains bounded by design.

## J. Shipping

- [ ] MANDATORY CI green without Instagram secrets.
- [ ] MANDATORY `cargo build --release` succeeds from clean clone with documented system deps.
- [ ] MANDATORY launcher works from clean documented setup.
- [ ] MANDATORY README describes unofficial Instagram risk and Linux/text-terminal requirements.
- [ ] MANDATORY CHANGELOG has v1.0.0.
- [ ] MANDATORY release archive contains expected files only.
