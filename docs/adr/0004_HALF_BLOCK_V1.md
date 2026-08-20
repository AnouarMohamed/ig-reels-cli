# ADR-0004 — v1.0 Uses Upper-Half Block Cells

**Status:** accepted

## Decision
Use `U+2580` (`▀`) for every video cell.

Top RGB sample -> 24-bit foreground.
Bottom RGB sample -> 24-bit background.

## Why
- two independent colors per character,
- doubles vertical sample density,
- with common ~2:1 cell H/W ratio, effective sample pixels are near square,
- algorithm is deterministic and fast enough to ship,
- photographic color fidelity is better than luminance ASCII ramps.

## Deferred
Quadrant/Braille/adaptive glyph selection can be separate future renderers only after v1.0.
