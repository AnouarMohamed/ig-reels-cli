# Locked Decisions

| ID | Decision | Status |
|---|---|---|
| D-001 | Python remains a separate Instagram gateway process | locked |
| D-002 | v1.0 platform target is Linux | locked |
| D-003 | v1.0 feed source is Reels/Discover only | locked |
| D-004 | UDS stream + 4-byte big-endian framing + MessagePack named maps | locked |
| D-005 | one request/one response per socket connection | locked |
| D-006 | Rust MessagePack structs serialize as named maps | locked |
| D-007 | `ffprobe` CLI JSON is used for media metadata | locked |
| D-008 | `ffmpeg` CLI is used for RGB24 video decoding | locked |
| D-009 | `mpv --vo=tct` is smoke-test-only | locked |
| D-010 | final video is Unicode/ANSI text cells only | locked |
| D-011 | terminal image protocols are forbidden in shipped renderer | locked |
| D-012 | v1.0 cell glyph is U+2580 `▀` | locked |
| D-013 | top RGB sample -> foreground, bottom RGB sample -> background | locked |
| D-014 | default render FPS is 15 | locked |
| D-015 | default max video width is 120 terminal columns and max video height is 60 terminal rows | locked |
| D-016 | fallback terminal cell height/width ratio is 2.0 | locked |
| D-017 | renderer builds complete frame in memory before output | locked |
| D-018 | v1.0 performs full-frame repaint; dirty-region diff is post-v1 | locked |
| D-019 | one display task is sole TUI stdout writer | locked |
| D-020 | Crossterm direct use; no Ratatui/TUI framework | locked |
| D-021 | rodio directly attempts MP4/AAC playback | locked pending audio spike |
| D-022 | audio playback position is preferred AV clock | locked pending audio spike |
| D-023 | App event loop is sole mutable navigation/state owner | locked |
| D-024 | one unseen downloaded prefetch | locked |
| D-025 | back+forward visited file cache max = 5 non-current Reels | locked |
| D-026 | no live re-decode/rescale on terminal resize during current Reel | locked |
| D-027 | gateway handles IPC serially in v1.0 | locked |
| D-028 | gateway `ping` is auth-free; auth initializes lazily | locked |
| D-029 | cache uses unique private per-run child directory | locked |
| D-030 | media redirect chain max = 5 and every hop must remain HTTPS | locked |
| D-031 | terminal input uses bounded-time poll/read worker | locked |
| D-032 | Cargo.lock committed; Python dependencies exactly pinned | locked |
| D-033 | no engagement/challenge/stealth automation | locked |
| D-034 | no FFmpeg FFI and no embedded Python in v1.0 | locked |
| D-035 | source geometry uses media probe + terminal-cell aspect correction | locked |
| D-036 | renderer quality changes must preserve text-cell-only invariant | locked |
| D-037 | display uses reliable control queue plus latest-value frame slot; control is prioritized | locked |

`D-021` and `D-022` are proof obligations. If the audio spike disproves them for real Reel files, stop and write a replacement ADR before changing architecture.
