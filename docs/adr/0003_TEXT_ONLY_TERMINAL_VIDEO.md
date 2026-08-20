# ADR-0003 — Text-Cell-Only Video Rendering

**Status:** accepted; product-defining

## Context
The desired experience is video that exists in the terminal as text output, not an inline image protocol masquerading as terminal rendering.

## Decision
The shipped renderer may use Unicode glyphs and ANSI/xterm-style color/cursor/control sequences only.

Forbidden:
- Kitty graphics protocol,
- Sixel,
- iTerm inline image protocol,
- external graphical player/window,
- embedded browser/Canvas,
- terminal-specific image libraries.

## Consequences
- fidelity is constrained by terminal cell grid/throughput,
- renderer engineering becomes a core feature,
- performance improvements must preserve text-only invariant.
