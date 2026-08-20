# 00 — Product Scope and Requirements

## 1. Product statement

IG-Reels-CLI v1.0 is a **personal, local, Linux terminal media client** for passively viewing a read-only Instagram Reels/Discover stream.

Its defining feature is a custom text-video renderer: full-color decoded video is represented using Unicode terminal cells and ANSI 24-bit foreground/background colors.

## 2. Primary user journey

1. User runs the project launcher.
2. Launcher establishes private runtime/state/cache/log paths.
3. Python gateway binds its Unix socket and answers `ping` without contacting Instagram.
4. Rust client pings gateway and performs local runtime preflight (`ffmpeg`, `ffprobe`, UTF-8 terminal, usable size).
5. Rust asks gateway for a Reel batch.
6. Gateway lazily reuses session or attempts one login initialization.
7. Rust downloads first Reel atomically.
8. Rust probes the media.
9. Rust freezes render geometry for that Reel.
10. FFmpeg decodes RGB24 frames at 15 FPS into a bounded pipe/channel.
11. Rodio starts audio if usable.
12. Rust converts frames into text cells and presents them through the single display owner.
13. While Reel N plays, Reel N+1 downloads.
14. User uses Next/Previous/Pause/Quit.
15. Quit cancels workers, kills children, restores terminal, deletes current run cache, and stops gateway through launcher ownership.

## 3. Functional requirements

### FR-001 Session reuse
Gateway persists session settings outside git and prefers session reuse over fresh login.

### FR-002 Read-only feed
Gateway performs only read operations necessary to obtain Reel metadata and URLs.

### FR-003 Stable DTO
Rust never depends on instagrapi model classes. Gateway normalizes upstream objects to IPC DTOs.

### FR-004 Metadata queue
Rust keeps a bounded unseen metadata queue and refills below a low-water mark.

### FR-005 Deduplication
A Reel id already current, visited, prefetched, queued, or seen in-session is not reinserted.

### FR-006 Atomic download
Only fully downloaded, validated final `.mp4` paths are cache-ready.

### FR-007 One-ahead prefetch
During Reel N playback, one unseen Reel should be downloading or ready when possible.

### FR-008 Probe before playback
Downloaded media is probed before custom playback. Geometry/audio presence are not guessed from filename or DTO.

### FR-009 Text-cell video
The shipped player renders video only as Unicode characters plus terminal control sequences.

### FR-010 Full color
The v1.0 renderer uses 24-bit foreground/background RGB values.

### FR-011 Aspect preservation
Displayed video must preserve source display aspect ratio within the documented tolerance.

### FR-012 Audio
Usable media audio plays through rodio when a valid output device exists.

### FR-013 Controls
- Next: `n`, `j`, Right
- Previous: `p`, `k`, Left
- Pause/resume: Space
- Quit: `q`

### FR-014 Navigation history
Back and forward visited stacks behave like browser history. After moving backward, Next retraces forward history before consuming unseen feed.

### FR-015 Status line
One sanitized line displays available `@username`, caption excerpt, like count, playback/audio state, and key hints.

### FR-016 Natural advance
Natural Reel end follows the same forward path as Next.

### FR-017 Resize safety
Resize never corrupts the terminal. Current Reel retains its frozen decode geometry; next Reel uses new dimensions.

### FR-018 Cleanup
Normal exit leaves no app-owned transient media for the current run and no child media processes.

## 4. Non-functional requirements

### NFR-001 Secret safety
No credentials/session/cookies/full signed media URLs in source, logs, panic text, fixtures, screenshots, or release archive.

### NFR-002 Bounded memory
Decoded RGB queue is bounded. Entire Reel is never buffered as raw frames.

### NFR-003 Bounded disk
Cache and file sizes are explicitly bounded.

### NFR-004 Bounded retry
No infinite retry loops.

### NFR-005 Terminal injection safety
Remote strings cannot emit terminal control sequences.

### NFR-006 Deterministic state
One App loop applies state transitions.

### NFR-007 Testability without Instagram
Most CI tests use fake gateway/local HTTP/generated media.

### NFR-008 Visible failure
No boundary failure is silently swallowed.

### NFR-009 Renderer identity
No implementation may satisfy a renderer test by delegating to an image protocol or external graphical player.

### NFR-010 Reproducibility
Pinned dependencies and committed lockfile make a clean-clone build reproducible within documented system prerequisites.

## 5. v1.0 operating constants

| Constant | Value |
|---|---:|
| IPC payload maximum | 1 MiB |
| gateway Reel request batch | 12 |
| metadata low-water | 4 unseen |
| metadata target | 12 unseen |
| downloaded unseen prefetch | 1 |
| visited non-current file history | 5 total across back+forward |
| max one Reel download | 150 MiB |
| HTTP connect timeout | 10 s |
| HTTP whole-request deadline | 60 s |
| redirect maximum | 5 |
| IPC connect timeout | 2 s |
| IPC request timeout | 30 s |
| render FPS | 15 |
| max video columns | 120 |
| max video rows | 60 |
| reserved status rows | 1 |
| fallback cell H/W aspect | 2.0 |
| accepted fallback clamp | 1.0–4.0 |
| decoded frame channel capacity | 3 frames |
| input poll timeout | 50 ms |
| minimum terminal | 20 cols × 8 rows |
| stale run-cache cleanup age | 24 h |
| FFmpeg stderr retained tail | max 64 KiB |

Constants live in one configuration/constants module; do not scatter literals.

## 6. Deliberate limitations

- Current Reel does not restart FFmpeg on resize.
- v1.0 renderer is `▀` half-block only.
- No dirty-region rendering in v1.0.
- No terminal capability downgrade to 256 colors; truecolor is required for release behavior.
- Truecolor is a documented requirement. Because capability discovery is imperfect, unknown-capability ANSI terminals continue with a warning; `TERM=dumb`/unusable terminals fail. No automatic 256-color downgrade.
- Pixel-level perfection is not expected because terminal font metrics differ; correct geometry uses measured metrics when available and fallback calibration otherwise.
