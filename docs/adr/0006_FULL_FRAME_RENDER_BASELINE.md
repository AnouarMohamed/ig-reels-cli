# ADR-0006 — Full-Frame Repaint for v1.0

**Status:** accepted

## Decision
Every presented frame contains the entire current video raster. The renderer may coalesce redundant color SGR commands and batch output, but does not diff against previous frame.

## Why
- correctness is much simpler,
- stale-cell behavior is predictable,
- avoids cursor-patch explosion/race bugs,
- performance is evaluated before adding complexity.

Dirty-region/delta repaint is a future explicit optimization, not agent improvisation.
