# 14 — Configuration and Dependency Policy

## 1. Shared environment

Launcher supplies:

```text
IG_REELS_SOCKET_PATH
IG_REELS_SESSION_PATH
IG_REELS_CACHE_DIR
IG_REELS_LOG_DIR
```

Gateway-only:

```text
IG_USERNAME
IG_PASSWORD
```

Optional renderer configuration is intentionally small in v1.0:

```text
IG_REELS_RENDER_FPS=15
IG_REELS_MAX_COLS=120
IG_REELS_MAX_ROWS=60
IG_REELS_CELL_ASPECT_FALLBACK=2.0
```

Values are validated and bounded. Release defaults match manifesto constants. Users do not need to set them.

## 2. No password CLI argument

Credentials never appear in process argv.

`.env` is development convenience only and gitignored. `.env.example` contains placeholders.

## 3. Baseline versions checked 2026-08-20

Python:

```text
instagrapi==2.18.16
msgpack==1.2.1
```

Rust conception baseline:

```text
tokio              1.53.1
reqwest             0.13.4
crossterm           0.29.0
rmp-serde           1.3.1
serde               1.0.229
rodio               0.22.2
tokio-util          0.7.19
thiserror           2.0.20
tracing             0.1.44
tracing-subscriber  0.3.23
```

Pins are starting points. The implementation task must still verify method signatures against the pinned crate source/docs or compiler.

## 4. System tools

Required final runtime:
- Python 3.10+,
- `ffmpeg`,
- `ffprobe`,
- UTF-8 locale,
- Linux audio stack supported by rodio/CPAL,
- ANSI truecolor-capable terminal.

Development smoke only:
- mpv with `tct` output.

## 5. Crossterm geometry fact

Crossterm terminal `size()` reports columns/rows. `window_size()` may expose pixel width/height, but current docs explicitly warn pixel dimensions may be unreliable or zero on Unix. Geometry therefore must implement fallback rather than depending on pixels.

## 6. Rodio proof obligation

Current rodio docs expose a `Player`, default device/sink builder path, MP4 among default common decoder formats, and `Player::get_pos()`. Exact construction calls are verified in the audio spike task. Do not copy old `OutputStream::try_default` examples from older rodio versions if they do not compile against the pin.

## 7. Reqwest redirect policy

Custom redirect policy must implement its own hop limit and HTTPS check. Do not assume the default loop limit applies after switching to a custom policy.

## 8. FFmpeg/ffprobe policy

Do not pin one FFmpeg major in Cargo. Treat tools as system dependencies and preflight at runtime.

At startup record versions to debug log if safe, but do not reject merely because version string differs unless a required command feature test fails.

## 9. Dependency upgrades

Any upgrade is its own task:
1. update pin,
2. update lock,
3. read changelog/docs,
4. compile,
5. run relevant tests,
6. update verified references,
7. ADR if behavior/architecture changes.
