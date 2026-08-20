# IG-REELS-CLI — BUILD MANIFESTO v3

## 0. Mission

Build a **local, single-user, read-only Linux Instagram Reels terminal player** whose video is rendered entirely as text cells.

The product must make a real Reel visually recognizable and fluid enough to watch while obeying a strict rendering constraint:

> The final player may output only UTF-8 text characters and terminal control sequences to the terminal. No terminal image protocol or graphical video surface is allowed.

The Rust text-video renderer is a primary engineering deliverable, not an implementation detail.

---

## 1. Absolute product invariants

1. **Text cells only.** Video pixels become Unicode glyphs plus ANSI colors.
2. **No Kitty graphics, Sixel, iTerm inline image protocol, sixel-compatible library, browser canvas, SDL window, OpenGL window, or GUI fallback.**
3. **No final-player delegation to mpv.** `mpv --vo=tct` is only an intermediate end-to-end proof.
4. **v1.0 renderer = upper-half block `▀` (`U+2580`) + truecolor foreground/background.**
5. **One terminal cell encodes exactly two vertical RGB samples** in the v1.0 renderer.
6. **Rust owns rendering, timing, input, navigation, cache, process lifecycle, and terminal restoration.**
7. **Python owns Instagram authenticated access only.**
8. **The app is read-only.** No like/comment/follow/save/post/share/DM automation.

---

## 2. Engineering invariants

1. One task card at a time.
2. Edit only paths explicitly allowed by the current card.
3. Never invent dependency APIs. Verify against pinned source/docs or compiler output.
4. Never expose credentials, session state, cookies, signed media URLs, or environment dumps.
5. Every network/process/filesystem/audio/terminal boundary has explicit error handling and cancellation behavior.
6. No broad refactors while solving a local task.
7. No `unsafe` in v1.0.
8. No runtime `unwrap()`/`expect()` at external boundaries.
9. CI must pass without Instagram credentials.
10. Exactly one component writes final TUI stdout.
11. Every async result that can become stale carries an operation/generation id.
12. Terminal restoration is mandatory on normal exit, handled error, Ctrl-C, and unwinding panic where feasible.
13. Remote captions/usernames are sanitized before display.
14. External programs are spawned as executable + argument vector, never `sh -c` or shell strings.
15. A failed mandatory architecture assumption triggers an ADR, not an improvised workaround.

---

## 3. v1.0 scope

### Included

- Linux local execution.
- One Instagram account/session.
- Read-only Reels/Discover feed.
- Session persistence in Python gateway.
- Versioned framed MessagePack IPC over Unix domain socket.
- Session-local Reel id deduplication.
- Atomic HTTPS media download.
- Unique per-run transient cache directory.
- One-ahead download prefetch.
- `ffprobe` media inspection.
- FFmpeg CLI RGB24 video decoding at a fixed render FPS.
- Terminal geometry/aspect-ratio correction.
- ANSI 24-bit upper-half-block text video.
- Rodio audio playback.
- Audio-master AV scheduling when audio is available.
- Next, previous, pause/resume, quit.
- Back/forward visited history.
- One sanitized status line.
- Structured file logs with URL/secret redaction.
- Deterministic cleanup.
- Release packaging and installation/run documentation.

### Explicitly excluded

- all write/engagement Instagram actions,
- hashtag/following/friends feed selector,
- multi-user server mode,
- database,
- web UI,
- persistent watch history,
- automatic challenge/checkpoint solving,
- anti-detection/proxy rotation/rate-limit evasion,
- Windows release target,
- macOS release acceptance,
- embedded Python/PyO3,
- FFmpeg FFI,
- terminal image protocols,
- live rescale of the already-playing FFmpeg decode stream,
- differential dirty-region renderer in v1.0,
- quadrant/braille renderer in v1.0,
- GPU acceleration.

---

## 4. Process architecture

```text
Instagram private/unofficial access
             |
             v
+---------------------------+
| Python IG gateway         |
| - instagrapi              |
| - session load/save       |
| - Reels/Discover fetch    |
| - DTO normalization       |
+-------------+-------------+
              |
              | framed MessagePack v1
              | Unix SOCK_STREAM
              v
+--------------------------------------------------+
| Rust terminal application                        |
|                                                  |
| feed -> download -> probe -> decode -> render    |
|                     |          |        |         |
|                     |          |        +-> ANSI stdout
|                     |          +-> RGB24 frames  |
|                     +-> ffprobe                  |
|                                                  |
| audio: rodio                                     |
| input: crossterm                                 |
| orchestration: tokio                             |
+--------------------------------------------------+
```

No application state crosses into Python. No Instagram credential crosses into Rust.

---

## 5. Locked v1.0 technology decisions

| Concern | Decision |
|---|---|
| Instagram | Python `instagrapi` |
| Python/Rust boundary | UDS stream + framed MessagePack |
| IPC frame | 4-byte big-endian length + MessagePack map |
| Rust runtime | Tokio |
| HTTP | Reqwest |
| media inspect | `ffprobe` CLI JSON |
| video decode | `ffmpeg` CLI -> RGB24 stdout |
| smoke playback | `mpv --vo=tct` only |
| terminal control | Crossterm |
| TUI framework | none |
| video renderer | Unicode `▀` + ANSI 24-bit FG/BG |
| default render rate | 15 FPS |
| default max video columns | 120 |
| default max video rows | 60 |
| cell aspect fallback | 2.0 terminal-cell height/width |
| audio | rodio |
| AV master | rodio playback position when usable audio exists |
| app state | one serialized App event loop |
| prefetch | exactly one unseen downloaded Reel ahead |
| visited cache | max 5 non-current visited Reel files across back+forward |
| release platform | Linux |

Changing one requires an ADR before code.

---

## 6. Text-video rendering invariant

For each cell:

```text
sample(x, 2y)     -> foreground RGB
sample(x, 2y + 1) -> background RGB
cell glyph        -> U+2580 '▀'
```

Conceptual terminal bytes:

```text
ESC[38;2;Rt;Gt;Btm   # top sample -> foreground
ESC[48;2;Rb;Gb;Bbm   # bottom sample -> background
UTF-8("▀")
```

The renderer may omit redundant SGR commands when color state is unchanged, but it may not alter the pixel-to-cell meaning.

The complete frame is assembled in memory before the display owner writes it.

---

## 7. Geometry invariant

Terminal cells are not assumed square.

Let:

```text
cell_ratio = physical_cell_height / physical_cell_width
```

Use terminal-reported pixel dimensions only when nonzero and plausible. Otherwise use the v1.0 fallback `2.0`. The value is clamped to a safe configured range.

Because one output cell represents two vertical samples:

```text
sample_physical_ratio = (cell_height / 2) / cell_width
                      = cell_ratio / 2
```

The decoded sample raster must be chosen so the displayed video preserves the source display aspect ratio under that sample geometry. Exact math is normative in `spec/04_MEDIA_PROBE_AND_GEOMETRY.md`.

---

## 8. Playback invariant

- FFmpeg emits constant-rate RGB frames at the geometry frozen at Reel start.
- RGB frame channel is bounded.
- If audio is usable, rodio playback position is the master presentation clock.
- Late video frames are dropped. Audio is not stalled to preserve every video frame.
- Without audio, a monotonic clock is used.
- Pause freezes presentation and pauses audio.
- Next/Previous/Quit cancels the current playback generation.
- Events from stale generations are ignored.

---

## 9. Terminal ownership invariant

After entering alternate/raw mode:

- input worker reads events,
- application loop owns state,
- playback workers produce typed render frames/events,
- **display owner is the only stdout writer**.

No logger, decoder, downloader, or App code writes to stdout while TUI mode is active.

---

## 10. Cache invariant

- Download to unique `.part` file.
- Rename into final `.mp4` only after success.
- Every run creates one private `run-*` directory.
- Normal shutdown deletes only its own run directory.
- Stale cleanup only targets validated old app-owned `run-*` directories.
- Never recursively delete the configured cache root itself.

---

## 11. Ship definition

v1.0 is not “done” when it compiles. It is shipped only after:

- protocol tests pass cross-language,
- fake gateway integration passes,
- real gateway manual smoke passes,
- real Reel download passes,
- local generated clip renders correctly in text mode,
- a human confirms recognizable motion/color/aspect in at least two supported terminals,
- real Reel audio works or the documented audio fallback is triggered correctly,
- AV scheduling survives a normal Reel without accumulating obvious drift,
- repeated navigation/pause/resize does not corrupt terminal state,
- 30-Reel soak completes without leaked children or unbounded disk/memory growth,
- cleanup and Ctrl-C restore terminal,
- release build and launcher work from a clean clone,
- CI passes without secrets,
- release archive contains no `.env`, session, cache, logs, fixtures with secrets, or credentials.

Detailed release gates are normative in `spec/13_ACCEPTANCE.md` and `spec/15_RELEASE_PACKAGING.md`.
