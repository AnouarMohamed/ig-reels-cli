# ADR-0005 — Correct for Terminal Cell Aspect Ratio

**Status:** accepted

## Decision
Derive physical terminal cell height/width ratio from reported pixel metrics when plausible; otherwise use fallback 2.0. Compute sample raster so displayed physical DAR matches source DAR.

Normative equation:

```text
sample_width / sample_height = source_DAR * cell_ratio / 2
```

## Consequence
Portrait video is not stretched merely because terminal cells are rectangular.
