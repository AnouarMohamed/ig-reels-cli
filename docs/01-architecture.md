System Architecture

```
┌─────────────────────────────────────┐   UDS + msgpack   ┌───────────────────────────────────────┐
│        Python Auth Daemon           │◄──────────────────►│            Rust TUI App               │
│  (instagrapi wrapper, session mgmt) │   request/response │   (terminal, render, input, etc.)     │
└─────────────────────────────────────┘                    └───────────────────────────────────────┘
        │                                                            │
        │ Instagram HTTPS API                                        │
        ▼                                                            ▼
┌─────────────────────────────────────┐                    ┌───────────────────────────────────────┐
│   Instagram Servers (private API)   │                    │   Local FFmpeg (frame/audio extract)    │
│   (video URLs, metadata)            │                    │   (invoked via rust-ffmpeg-next or CLI) │
└─────────────────────────────────────┘                    └───────────────────────────────────────┘
                                                            │
                                                            ▼
                                                    ┌─────────────────────┐
                                                    │  tmp-cache/ (.mp4)  │
                                                    └─────────────────────┘
```

Component Table:

| Component                     | Responsibility                                               | Language | Key Dependencies                     |
|-------------------------------|--------------------------------------------------------------|----------|--------------------------------------|
| Python Auth Daemon            | Login to Instagram, maintain session, fetch reel batches     | Python   | instagrapi, msgpack, stdlib socket   |
| ig_client.py                  | Thin wrapper around instagrapi for daemon use                | Python   | instagrapi                           |
| daemon.py                     | UDS server, msgpack request/response handling                | Python   | msgpack, socket, ig_client           |
| Rust TUI App (main.rs)        | Entry point, sets up Tokio runtime, wires modules            | Rust     | tokio, crossterm                     |
| ipc.rs                        | UDS client, msgpack encode/decode for requests/responses    | Rust     | tokio-unix, tokio-util, msgpack      |
| fetch.rs                      | Calls IPC to get reel batches, manages incoming reel queue   | Rust     | ipc.rs, async streams                |
| download.rs                   | Downloads MP4 video to tmp-cache, implements prefetch logic | Rust     | reqwest or tokio::fs, tempfile       |
| decode.rs                     | Extracts RGB video frames from MP4 via ffmpeg                | Rust     | ffmpeg-next (or subprocess ffmpeg)   |
| render.rs                     | Converts RGB frames to terminal output (ANSI truecolor/Kitty)| Rust     | crossterm (for size, raw mode)       |
| audio.rs                      | Extracts and plays audio track via rodio, syncs to video    | Rust     | rodio, rubato (optional resampler)   |
| input.rs                      | Captures keypresses in raw mode (crossterm)                  | Rust     | crossterm                            |
| ui.rs                         | Renders thin status line (username, caption, like count)     | Rust     | crossterm                            |
| app.rs                        | Top-level state machine: coordination, prefetch, playback   | Rust     | all above modules                    |
| tmp-cache/                    | Transient storage for downloaded MP4 files (gitignored)      | N/A      | tmpfs or disk cleanup on exit        |

Process Boundaries:

1. **Python ↔ Instagram** – HTTPS API calls via instagrapi library (network call).
2. **Python Daemon ↔ Rust App** – Unix Domain Socket (UDS) using msgpack encode/decode (IPC boundary).
3. **Rust App ↔ FFmpeg** – Either via Rust ffmpeg-next crate (in-process FFI) or subprocess calling ffmpeg CLI (process boundary). The manifesto indicates either approach; we treat as a process boundary for safety.
4. **Rust App ↔ Local File System** – Reading/writing MP4 files in tmp-cache/ (in-process, but crosses to storage).
5. **Rust App ↔ Audio Hardware** – Via rodio (process boundary to sound subsystem).
6. **Rust App ↔ Terminal** – Via crossterm (process boundary to TTY).