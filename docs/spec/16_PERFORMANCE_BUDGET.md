# 16 — Performance Budget

Performance must be measured, but the release has explicit initial budgets so the agent cannot optimize randomly.

## 1. Default operating point

```text
FPS = 15
MAX_VIDEO_COLS = 120
MAX_VIDEO_ROWS = 60
STATUS_ROWS = 1
FRAME_CHANNEL_CAPACITY = 3
```

For a typical 9:16 Reel in a 120×40 terminal with ratio 2.0, video content is roughly 44×39 terminal cells (~1716 glyphs/frame).

Landscape can approach ~120×34 cells (~4080 glyphs/frame).

## 2. Raw RGB memory

At absolute configured max sample raster 120×120 (60 half-block rows):

```text
120 * 120 * 3 = 43,200 bytes/frame
3-frame channel ~= 126.6 KiB plus Vec overhead
```

Typical portrait playback in a 120×40 terminal is much smaller (~44×78 samples). Both width and row caps prevent oversized terminals from creating unbounded renderer load.

## 3. ANSI byte volume

Worst case every cell changes both FG/BG and requires control bytes. This can be tens of bytes per cell. Do not assume glyph count equals output byte count.

Measure:
- encoded bytes/frame,
- encode time/frame,
- write time/frame.

## 4. CPU budget

On the primary development/acceptance machine, target:
- median RGB->ANSI encode < 50% of 66.7 ms frame interval,
- p95 encode < 66.7 ms for default max geometry,
- app remains responsive to input while rendering.

These are practical v1 gates, not universal hardware guarantees.

## 5. Frame-drop rule

If rendering/terminal output falls behind audio clock, drop video frames. Never grow the queue to catch up later.

## 6. Optimization order if budget fails

Allowed order:
1. verify release build, not debug,
2. remove accidental per-cell allocations/format! overhead,
3. reserve/reuse output buffer safely,
4. coalesce redundant FG/BG SGR changes,
5. batch to one/few stdout writes per frame,
6. profile terminal write bottleneck,
7. lower max columns or FPS only through explicit documented decision/ADR if necessary,
8. consider dirty-region renderer only as a separate scoped post-baseline change.

Forbidden response:
- switch to Kitty/Sixel/image protocol,
- unbounded buffering,
- sacrifice terminal restoration/input responsiveness,
- silently reduce colors to 256 without product decision.

## 7. Performance artifacts

Keep benchmark results in developer notes or release QA record, not hard-coded machine claims in README.

The shipped default is justified by visual usability + bounded behavior, not by pretending every terminal reaches exactly 15 presented frames/sec.
