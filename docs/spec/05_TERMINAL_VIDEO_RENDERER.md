# 05 — Terminal Text-Video Renderer

This is a core product specification.

## 1. Renderer contract

Input:

```text
RgbFrame {
  width: u16/u32,
  height: u16/u32,     # MUST be even
  bytes: contiguous RGB24, row-major, length = width*height*3
  frame_index: u64
}

RenderGeometry {
  sample_width,
  sample_height,
  cell_cols = sample_width,
  cell_rows = sample_height / 2,
  origin_x,
  origin_y
}
```

Output:

```text
RenderFrame {
  playback_generation: u64,
  frame_index: u64,
  bytes: Vec<u8>       # UTF-8 + ANSI terminal control only
}
```

The renderer is a pure transformation. It does not write stdout.

## 2. Exact RGB indexing

For zero-based `(x, y)`:

```text
index = (y * width + x) * 3
r = bytes[index]
g = bytes[index + 1]
b = bytes[index + 2]
```

Bounds must be validated once at frame boundary, not with panicking unchecked assumptions.

Expected length:

```text
expected = width * height * 3
```

Use checked arithmetic before comparing/allocating.

## 3. One cell mapping

For output cell `(cx, cy)`:

```text
top = RGB(cx, cy*2)
bottom = RGB(cx, cy*2 + 1)
glyph = U+2580 '▀'
foreground = top
background = bottom
```

This mapping never changes in v1.0.

## 4. ANSI SGR encoding

Truecolor foreground semantic sequence:

```text
ESC [ 38 ; 2 ; R ; G ; B m
```

Truecolor background semantic sequence:

```text
ESC [ 48 ; 2 ; R ; G ; B m
```

Reset semantic sequence at safe boundaries:

```text
ESC [ 0 m
```

Cursor positioning should use Crossterm commands or a verified equivalent generated into the frame buffer. Do not concatenate remote text into escape sequences.

## 5. Color-state coalescing

The encoder maintains local state while building a frame:

```text
current_fg: Option<Rgb>
current_bg: Option<Rgb>
```

Before each glyph:
- emit FG sequence only if top RGB differs from current_fg,
- emit BG sequence only if bottom RGB differs from current_bg,
- emit glyph.

At start of each frame, treat color state as unknown unless DisplayOwner explicitly guarantees inherited state. v1.0 safest policy: renderer begins with reset and explicit first colors.

At end of video region, emit reset before UI/status.

Do not approximate/quantize colors in v1.0.

## 6. Row placement

For each cell row `cy`:
1. move cursor to `(origin_x, origin_y + cy)`,
2. encode exactly `cell_cols` glyphs,
3. do not emit newline because newline semantics depend on terminal mode/wrap,
4. next row begins with explicit cursor move.

Line wrapping should be disabled while TUI is active. Renderer still must never rely on wrapping.

## 7. Clearing old content

When a new playback generation has smaller geometry than the previous generation, stale cells can remain.

DisplayOwner therefore handles `BeginGeneration`:
1. reset attributes,
2. clear the video viewport/alternate screen once,
3. store new generation and geometry,
4. then accept frames for that generation.

Do **not** clear whole screen every video frame.

Resize-to-too-small state may clear once before displaying a small message.

## 8. Full-frame baseline

v1.0 sends every cell of every presented frame.

Forbidden in v1.0 core implementation unless a later explicit task/ADR adds it:
- dirty rectangle diff,
- per-cell previous-frame suppression,
- color quantization,
- braille,
- quadrant glyph search,
- image protocols.

This keeps correctness auditable.

## 9. Frame allocation

Avoid per-cell heap allocations.

Recommended:
- `Vec<u8>` with estimated capacity,
- append ASCII escape bytes and UTF-8 glyph bytes,
- reuse renderer scratch buffer only if ownership is simple and tests prove no aliasing/stale mutation.

A conservative worst-case capacity estimate may use ~50 bytes/cell plus cursor row overhead. It is an estimate, not a correctness limit.

## 10. Status line

Status is rendered outside RGB renderer or appended by a UI encoder after reset.

Rules:
- exactly one terminal row,
- sanitize remote text,
- collapse tabs/newlines to spaces,
- remove C0/C1/ESC/control chars,
- truncate by visible character width policy,
- key hints are trusted static text,
- never permit remote ANSI.

Initial v1 truncation may count Unicode scalar values rather than grapheme display width if documented; if this causes layout bugs, add a dedicated Unicode-width task rather than improvising.

## 11. Test vectors

### 1×2 sample -> 1 cell
Input RGB bytes:

```text
top    = (255,0,0)
bottom = (0,0,255)
```

Semantic expected:
- FG red,
- BG blue,
- `▀`.

Test should parse/assert semantic byte fragments, not depend unnecessarily on one Crossterm implementation's cursor sequence.

### 2×2 sample -> 2 cells
Pixels row 0: red, green
Pixels row 1: blue, white

Expected cells:
1. FG red / BG blue / `▀`
2. FG green / BG white / `▀`

### uniform frame
All samples same RGB. Encoder should emit one FG and one BG color establishment per row/frame policy, not one pair per cell. Exact count is golden-tested once implementation policy is frozen.

## 12. Renderer invariants tests

Mandatory unit tests:
- rejects odd sample height,
- rejects byte-length mismatch,
- checked multiplication overflow path,
- correct RGB indexing,
- correct 1×2 mapping,
- correct 2×2 mapping,
- no newline-based row movement,
- no image-protocol bytes/signatures,
- remote status ESC stripped,
- output is valid UTF-8 wherever interpreted as text sequences/glyphs,
- color reset occurs before status.

## 13. Visual acceptance

Generated fixture must show:
- moving geometry,
- red/green/blue regions,
- gradients,
- diagonal edges,
- human-readable test pattern/text in the source video if practical.

Human checks:
- colors approximately correct,
- video recognizable,
- portrait not stretched,
- no persistent stale rows between Reels,
- no cursor visible during playback,
- no scrolling,
- no frame text leaking after quit.
