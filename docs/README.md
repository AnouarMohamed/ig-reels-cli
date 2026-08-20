# IG-Reels-CLI — Terminal Video Shipping Specification v3

This folder is the **implementation contract** for IG-Reels-CLI v1.0.

The defining product constraint is now explicit:

> **Reels are rendered as terminal text cells. The final player may emit UTF-8 Unicode characters and ANSI/ECMA/xterm-style terminal control sequences only. It must never use a terminal image protocol, embedded graphical surface, browser canvas, sixel, Kitty graphics, iTerm inline images, or a separate video window.**

The project is therefore not merely an Instagram CLI. Its technical centerpiece is a real-time **text-cell video engine** implemented in Rust.

## Product in one sentence

A local Linux terminal application that obtains a read-only Instagram Reels/Discover feed through an isolated Python gateway, downloads transient MP4 files, decodes them with FFmpeg, converts RGB frames into ANSI truecolor Unicode half-block cells, plays audio with rodio, supports deterministic navigation, and cleans up completely on exit.

## What changed from v2

The previous specification treated ANSI rendering as one rendering path and left higher-fidelity terminal graphics as a possible later path. That is no longer the product.

v3 makes these changes:

- **Text-cell rendering is the permanent product identity.**
- Kitty/Sixel/iTerm image protocols are explicitly forbidden for the shipped renderer.
- The renderer has its own normative data model, geometry, aspect-ratio math, byte encoder, performance budget, and golden tests.
- Terminal cell aspect ratio is accounted for. The implementation uses pixel dimensions when trustworthy and a documented fallback ratio otherwise.
- `ffprobe` is added as a required media-inspection tool so geometry and audio presence are not guessed.
- FFmpeg emits a fixed-size RGB24 sample raster whose dimensions are computed by Rust for the terminal.
- v1.0 uses the Unicode upper-half block `▀` (`U+2580`) with 24-bit foreground/background color: one terminal cell represents two vertical RGB samples.
- Full-frame rendering is the v1.0 correctness baseline. Differential repainting is intentionally out of the release gate.
- A render frame is built completely in memory and written by one exclusive display owner.
- The entire implementation path is decomposed into small task cards from empty repository through release archive.
- Release gates distinguish unit/integration tests, visual terminal checks, real-account checks, soak tests, and packaging checks.

## Source-of-truth precedence

When files disagree, use this order:

1. `MANIFESTO_V3.md`
2. `spec/*.md`
3. `DECISIONS.md`
4. `RISK_REGISTER.md`
5. `adr/*.md`
6. `agent/AGENT_RULES.md`
7. `agent/TASK_CARDS.md`
8. historical material under `source/`

If two files at the same precedence level conflict, **stop and fix the documentation before changing code**.

## Required reading before coding

Read in this exact order:

1. `MANIFESTO_V3.md`
2. `DECISIONS.md`
3. `RISK_REGISTER.md`
4. `spec/00_PRODUCT_SCOPE.md`
5. `spec/01_ARCHITECTURE.md`
6. `spec/02_IPC_PROTOCOL.md`
7. `spec/03_FEED_QUEUE_CACHE.md`
8. `spec/04_MEDIA_PROBE_AND_GEOMETRY.md`
9. `spec/05_TERMINAL_VIDEO_RENDERER.md`
10. `spec/06_PLAYBACK_AUDIO_SYNC.md`
11. `spec/07_APP_STATE_MACHINE.md`
12. `spec/08_SECURITY_AND_SAFETY.md`
13. `spec/09_ERRORS_AND_OBSERVABILITY.md`
14. `spec/10_TEST_STRATEGY.md`
15. `spec/11_BUILD_AND_SHIP_PLAN.md`
16. `spec/12_REPO_LAYOUT.md`
17. `spec/13_ACCEPTANCE.md`
18. `spec/14_CONFIGURATION_AND_DEPENDENCIES.md`
19. `spec/15_RELEASE_PACKAGING.md`
20. `spec/16_PERFORMANCE_BUDGET.md`
21. `spec/17_TERMINAL_COMPATIBILITY.md`
22. `spec/18_INTERNAL_EVENT_AND_DISPLAY_CONTRACTS.md`
23. `agent/AGENT_RULES.md`
24. `agent/COMMANDS.md`
25. **one and only one** task card from `agent/TASK_CARDS.md`

## How to use with a weak coding agent

Do not ask it to “build the app.” For each task give it:

- `agent/WEAK_MODEL_PROMPT.md`,
- `agent/AGENT_RULES.md`,
- `agent/COMMANDS.md`,
- exactly one task card,
- only the spec/ADR files listed by that card,
- only the existing source files it is allowed to edit,
- exact compiler/test output from the previous attempt.

The agent must list edit paths before touching code. If it needs another file, it must stop and say why.

## Shipped v1.0 renderer

The release renderer is intentionally simple and deterministic:

```text
MP4
  -> ffprobe metadata
  -> geometry calculation
  -> ffmpeg 15 FPS RGB24 sample frames
  -> pair two vertical RGB samples
  -> ANSI FG(top) + ANSI BG(bottom) + "▀"
  -> one in-memory frame buffer
  -> one display owner
  -> terminal
```

This is **not ASCII-art luminance rendering**. Each cell can carry two independent 24-bit RGB colors, so the visual result is low-resolution full-color video represented entirely by text cells.

## External runtime requirements

- Linux
- UTF-8 locale
- Python 3.10+
- `ffmpeg`
- `ffprobe`
- terminal supporting ANSI cursor control and 24-bit color
- working Linux audio backend/device for audio acceptance

Intermediate smoke milestone only:

- `mpv` with `--vo=tct`

## Release target

v1.0 is shipped only when every mandatory item in `spec/13_ACCEPTANCE.md` and `spec/15_RELEASE_PACKAGING.md` passes.
