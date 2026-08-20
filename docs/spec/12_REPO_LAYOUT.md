# 12 — Target Repository Layout

```text
ig-reels-cli/
├── README.md
├── LICENSE
├── CHANGELOG.md
├── .gitignore
├── .env.example
├── docs/
│   ├── architecture.md
│   ├── security.md
│   └── troubleshooting.md
├── scripts/
│   ├── run.sh
│   ├── bootstrap-dev.sh
│   ├── fake-gateway.py
│   ├── make-test-clips.sh
│   ├── test-protocol-contract.sh
│   └── release-check.sh
├── py-ig-gateway/
│   ├── requirements.txt
│   ├── config.py
│   ├── protocol.py
│   ├── ig_client.py
│   ├── daemon.py
│   └── tests/
│       ├── test_protocol.py
│       ├── test_ig_mapping.py
│       └── test_daemon.py
└── rust-tui/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── src/
    │   ├── main.rs
    │   ├── config.rs
    │   ├── error.rs
    │   ├── logging.rs
    │   ├── ipc.rs
    │   ├── model.rs
    │   ├── feed.rs
    │   ├── cache.rs
    │   ├── download.rs
    │   ├── media_probe.rs
    │   ├── geometry.rs
    │   ├── decode.rs
    │   ├── renderer.rs
    │   ├── ansi.rs
    │   ├── audio.rs
    │   ├── scheduler.rs
    │   ├── terminal.rs
    │   ├── input.rs
    │   ├── display.rs
    │   ├── ui.rs
    │   ├── app.rs
    │   └── shutdown.rs
    └── tests/
        ├── ipc_contract.rs
        ├── download_local.rs
        ├── media_pipeline.rs
        └── app_integration.rs
```

## Ownership boundaries

### `renderer.rs`
Pure RGB frame -> semantic terminal-cell/render-frame conversion. No stdout. No FFmpeg. No app navigation.

### `ansi.rs`
ANSI/terminal byte encoding helpers and color-state encoder. No remote unsanitized text.

### `display.rs`
Sole stdout writer in TUI mode. Generation filtering, BeginGeneration clear, frame/status writes.

### `geometry.rs`
Pure math only. No terminal I/O. Inputs are already measured metrics + media aspect.

### `media_probe.rs`
ffprobe process boundary + JSON normalization only. No rendering.

### `decode.rs`
ffmpeg RGB24 child + whole-frame reader only. No ANSI.

### `scheduler.rs`
Frame timing/drop policy only. No navigation ownership.

### `app.rs`
State machine/orchestration. No raw ANSI bytes.

### `terminal.rs`
RAII setup/restore and metric query. No media logic.

## Anti-patterns forbidden by layout

Do not create:
- `utils.rs` dumping ground for unrelated logic,
- monolithic `player.rs` containing process+audio+render+input+state,
- Python media downloader,
- renderer that directly calls stdout,
- App that directly executes FFmpeg command strings.
