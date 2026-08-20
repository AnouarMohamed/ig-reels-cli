# 04 — Media Probe and Terminal Geometry

This specification exists because a weak implementation will otherwise stretch portrait video, guess audio presence, or treat terminal cells as square pixels.

## 1. Media probe

Use `ffprobe` as a separate process before custom playback.

Conceptual invocation shape:

```text
ffprobe
  -v error
  -select_streams v:0
  -show_entries stream=width,height,sample_aspect_ratio,display_aspect_ratio
  -of json
  <file>
```

Audio presence may be queried in the same or a second bounded JSON probe. The implementation must verify exact local `ffprobe` syntax during its dedicated task.

Do not parse human-formatted stderr output.

## 2. `MediaInfo` required normalized fields

```text
video_width: u32 > 0
video_height: u32 > 0
display_aspect_ratio: positive rational/f64
audio_present: bool
duration: optional positive duration
```

If `display_aspect_ratio` is absent/invalid, derive using width, height, and valid sample-aspect-ratio; if SAR is unavailable, fall back to width/height.

Reject impossible dimensions or nonfinite/nonpositive aspect ratio.

Rotation/orientation must match what FFmpeg will output. The probe task must test at least one rotated fixture if FFmpeg/ffprobe metadata exposes rotation separately. If the selected FFmpeg invocation autorotates but `MediaInfo` does not account for it, geometry is wrong and the task is not complete.

## 3. Terminal metrics

Input:

```text
cols: u16
rows: u16
pixel_width: u16/u32 or 0
pixel_height: u16/u32 or 0
```

Crossterm `window_size()` may report pixel dimensions as zero/unreliable on Unix, so pixel dimensions are opportunistic, not mandatory.

### Measured cell ratio

If all are true:
- cols > 0,
- rows > 0,
- pixel_width > 0,
- pixel_height > 0,
- derived cell width/height are finite and positive,

then:

```text
cell_width_px  = pixel_width / cols
cell_height_px = pixel_height / rows
measured_ratio = cell_height_px / cell_width_px
```

Accept measured ratio only if `1.0 <= ratio <= 4.0`.

Otherwise use fallback `2.0`.

The normalized `cell_ratio` is always within `[1.0, 4.0]`.

## 4. Available video area

```text
available_cols = terminal_cols
available_rows = terminal_rows - STATUS_ROWS
max_cell_cols  = min(available_cols, MAX_VIDEO_COLS)
max_cell_rows  = min(available_rows, MAX_VIDEO_ROWS)
```

Minimum terminal before playback: 20 cols × 8 rows.

A half-block cell represents two sample rows, therefore:

```text
max_sample_width  = max_cell_cols
max_sample_height = 2 * max_cell_rows
```

`sample_height` must be even.

## 5. Aspect-ratio math

Definitions:

```text
source_dar = source_display_width / source_display_height
cell_ratio = physical_cell_height / physical_cell_width
```

One RGB sample displayed through half-block has physical dimensions:

```text
sample_width_physical  = cell_width
sample_height_physical = cell_height / 2
```

Therefore physical sample H/W ratio is:

```text
sample_h_over_w = cell_ratio / 2
```

For a sample raster `W x H`, displayed physical aspect is:

```text
displayed_dar = (W * cell_width) / (H * cell_height / 2)
              = (W / H) * (2 / cell_ratio)
```

To preserve source DAR:

```text
W / H = source_dar * cell_ratio / 2
```

Define:

```text
target_sample_ratio = source_dar * cell_ratio / 2
```

This equation is normative.

## 6. Largest fitting content rectangle

Inputs:
- `max_sample_width >= 1`,
- `max_sample_height >= 2`,
- `target_sample_ratio > 0`.

First normalize:

```text
max_h_even = largest even integer <= max_sample_height
```

If `max_h_even < 2`, geometry is unusable.

### Deterministic rounding helper

`nearest_even(v, max_h_even)`:
1. `lower = floor(v)`; if odd, subtract 1.
2. `upper = lower + 2`.
3. discard candidates `< 2` or `> max_h_even`.
4. choose candidate with smallest absolute distance to `v`.
5. on an exact tie, choose the **lower** candidate.

For ordinary integer width rounding use positive `round()` semantics (nearest integer; `.5` rounds upward for these positive values), then clamp to `[1, max_sample_width]`.

### Rectangle algorithm

```text
raw_h = max_sample_width / target_sample_ratio

if raw_h <= max_h_even:
    sample_width = max_sample_width
    sample_height = nearest_even(raw_h, max_h_even)
else:
    sample_height = max_h_even
    raw_w = sample_height * target_sample_ratio
    sample_width = round(raw_w)
    sample_width = clamp(sample_width, 1, max_sample_width)
```

Then assert:

```text
1 <= sample_width <= max_sample_width
2 <= sample_height <= max_h_even
sample_height % 2 == 0
```

No later code may independently round or modify these dimensions. The exact `sample_width` and `sample_height` are passed to FFmpeg and renderer.

The renderer does not need a full-screen black RGB raster. FFmpeg emits exactly the content raster; terminal renderer centers it with text-space padding. This reduces raw decode bandwidth.

## 7. Terminal cell dimensions of content

```text
video_cell_cols = sample_width
video_cell_rows = sample_height / 2
```

Center origin:

```text
origin_x = floor((available_cols - video_cell_cols) / 2)
origin_y = floor((available_rows - video_cell_rows) / 2)
```

All coordinates are zero-based in internal model. Convert to terminal API coordinate conventions only at the output boundary.

## 8. Example: 9:16 Reel, 120×40 terminal

Assume:
- status rows = 1,
- available rows = 39,
- max sample = 120×78,
- source DAR = 9/16 = 0.5625,
- cell ratio = 2.0.

Then:

```text
target_sample_ratio = 0.5625 * 2 / 2 = 0.5625
```

Height-limited:

```text
sample_height = 78
sample_width = round(78 * 0.5625) = round(43.875) = 44
```

Therefore this golden case is exactly **44 × 78 samples**, displayed as **44 × 39 terminal cells**.

## 9. Geometry tolerance test

For pure geometry tests, compute predicted displayed DAR:

```text
predicted = (W / H) * (2 / cell_ratio)
relative_error = abs(predicted - source_dar) / source_dar
```

Mandatory relative error <= 3% when dimensions are large enough (>=20 cells in the limiting direction). Very tiny terminals may exceed this due integer rounding and are covered by minimum-size policy.

## 10. Resize

Geometry is frozen per playback generation.

Resize updates `latest_terminal_metrics` in App. It does not mutate current `RenderGeometry`.

Display owner must clip/suspend safely if current content no longer fits. Next Reel recalculates geometry.
