# RELEASE QA — v1.0.0

Date:
Commit/tag candidate:
Linux distribution/kernel:
Terminal 1 + version:
Terminal 2 + version:
Python version:
ffmpeg/ffprobe version:
Rust version:

## Automated commands

| Command | Result | Notes |
|---|---|---|
| Python tests | NOT RUN | |
| protocol contract | NOT RUN | |
| cargo fmt --check | NOT RUN | |
| cargo check --locked | NOT RUN | |
| cargo test --locked | NOT RUN | |
| cargo clippy --locked --all-targets -- -D warnings | NOT RUN | |
| cargo build --release --locked | NOT RUN | |
| release-check.sh | NOT RUN | |

## Visual generated-media checks

| Check | Terminal 1 | Terminal 2 | Notes |
|---|---|---|---|
| portrait aspect | NOT RUN | NOT RUN | |
| landscape aspect | NOT RUN | NOT RUN | |
| colors/gradient | NOT RUN | NOT RUN | |
| no scrolling | NOT RUN | NOT RUN | |
| cursor restored | NOT RUN | NOT RUN | |
| pause/resume | NOT RUN | NOT RUN | |
| resize safety | NOT RUN | NOT RUN | |

## Live Instagram checks

| Check | Result | Notes (never paste secret/full URL) |
|---|---|---|
| session reuse | NOT RUN | |
| Reels batch | NOT RUN | |
| Rust direct media download | NOT RUN | |
| real text-video render | NOT RUN | |
| real audio | NOT RUN | |
| navigation/prefetch | NOT RUN | |
| 30-Reel soak | NOT RUN | |

## Post-soak

- Project ffmpeg children: NOT CHECKED
- Project mpv children: NOT CHECKED
- Current run cache removed: NOT CHECKED
- Terminal restored: NOT CHECKED
- Secret/URL log scan: NOT CHECKED
- Memory/disk bounded observation: NOT CHECKED

## Acceptance checklist

Copy every mandatory checkbox from `spec/13_ACCEPTANCE.md` here and attach evidence.

**SHIP DECISION:** DO NOT SHIP until every mandatory item is PASS.
