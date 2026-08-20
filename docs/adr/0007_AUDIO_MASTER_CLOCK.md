# ADR-0007 — Prefer Audio Playback Position as AV Master Clock

**Status:** accepted pending implementation spike

## Decision
When usable audio exists, use current rodio Player playback position as visual presentation clock. Drop late video frames rather than delaying audio.

Without audio, use monotonic clock.

## Stop condition
If real Reel files or rodio position behavior make this unreliable, write replacement ADR before architecture change.
