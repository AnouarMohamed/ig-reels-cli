# ADR-0002 — Use FFmpeg/ffprobe CLI, Not FFI

**Status:** accepted

## Decision
Use `ffprobe` JSON for media metadata and `ffmpeg` stdout rawvideo for decoded RGB24 frames.

## Why
- avoids C/FFmpeg FFI/build complexity,
- process failures are visible and cancellable,
- exact RGB frame boundary is simple,
- weak-agent implementation is easier to verify.

## Consequences
- system tools are runtime dependencies,
- child lifecycle/stderr draining are mandatory,
- startup preflight checks tool availability.
