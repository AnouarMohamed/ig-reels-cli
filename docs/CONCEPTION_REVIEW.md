# Conception Review — Why v3 Exists

## 1. The new idea changes the project's center of gravity

The earlier design could be described as “Instagram Reels in a terminal, eventually with a custom renderer.”

The corrected design is:

> **a real-time terminal text-video engine, with Instagram Reels as its media source.**

That distinction matters. It makes the renderer a product invariant rather than an interchangeable implementation detail.

## 2. Corrections made in v3

### A. “ANSI renderer” was too vague
A weak agent could interpret ANSI as colored ASCII, 256-color fallback, Kitty image payloads, or simply shelling to mpv.

**v3:** exact glyph (`U+2580`), exact two-sample mapping, truecolor SGR semantics, and forbidden graphical protocols.

### B. Terminal cells are not square
Naively decoding a 9:16 Reel into `cols × rows*2` assumes a cell is exactly twice as tall as wide. Often close, not guaranteed.

**v3:** derive cell H/W ratio from terminal pixel metrics only when plausible, otherwise use fallback 2.0. Geometry equation is explicit and tested.

### C. Media aspect was guessed
Without a probe layer, width/height/audio/orientation become ad-hoc assumptions inside FFmpeg/audio code.

**v3:** add `ffprobe` -> normalized `MediaInfo` before playback.

### D. Renderer/data/process concerns were mixed
A module that reads FFmpeg, converts pixels, writes stdout, and handles status becomes impossible to unit-test cleanly.

**v3:** separate `media_probe`, `geometry`, `decode`, `renderer`, `ansi`, `display`, `scheduler`.

### E. “Optimize ANSI” was underspecified
A weak model may implement fragile dirty diffing before the first correct frame exists.

**v3:** full-frame repaint is the v1.0 baseline; only color-state coalescing and buffered writes are allowed baseline optimizations.

### F. Quality fallback could destroy product identity
If ANSI throughput or quality disappoints, an agent might add Kitty/Sixel.

**v3:** image protocols are a hard violation. Performance failures are solved within the text-cell renderer or through explicit FPS/column decisions.

### G. Aspect correction needs deterministic rounding
Without a pure geometry function, tiny floating-point differences can alter FFmpeg dimensions and tests.

**v3:** one pure largest-fitting-rectangle algorithm, even sample height, golden tests, <=3% physical DAR error for normal-size cases.

### H. Stale screen cells between different Reel sizes
If Reel A is wider/taller than Reel B and full screen is never cleared, old colored cells remain.

**v3:** `DisplayOwner::BeginGeneration` semantics clear viewport/screen once per generation, not per frame.

### I. stdout ownership becomes even more important
ANSI video is high-volume output. Any log/status write interleaving corrupts control sequences.

**v3:** single display owner is mandatory and generation-aware.

### J. “Looks like video” needs a visual test corpus
Unit tests alone cannot prove visual recognizability.

**v3:** deterministic generated AV fixtures plus human visual acceptance on at least two terminal environments.

## 3. What remains intentionally simple

- 15 FPS default.
- 120-column maximum.
- One `▀` renderer.
- Full-frame repaint.
- No live rescale of the active decoder.
- No persistent DB/history.
- No multiple feed modes.

These are not missing sophistication. They keep the first release shippable.

## 4. Portfolio value

The strongest technical story becomes:

- process-isolated unstable external API adapter,
- custom framed IPC,
- bounded async download/decode pipeline,
- media metadata/probe handling,
- terminal geometry and physical aspect correction,
- custom real-time RGB-to-Unicode renderer,
- terminal escape-sequence optimization,
- audio-master scheduling/frame dropping,
- deterministic state machine/cancellation/cleanup.

That is a coherent systems project rather than a collection of libraries.
