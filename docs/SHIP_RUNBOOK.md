# v1.0 Ship Runbook

Use after implementation reaches T-079.

## 1. Freeze
- no dependency upgrades,
- no new renderer modes,
- no feed features,
- only release-blocking fixes.

## 2. Automated proof
Run release-check, Python tests, protocol contract, Rust fmt/check/test/clippy, release build.

## 3. Local generated-media proof
- generate portrait + landscape fixtures,
- custom renderer playback,
- audio fixture,
- pause/resume,
- resize,
- q and Ctrl-C cleanup.

## 4. Live proof
- gateway session reuse,
- real batch,
- real download,
- real custom text-video + audio,
- navigation/prefetch,
- 30-Reel soak.

## 5. Post-run inspection
- no project ffmpeg/mpv children,
- no current run cache,
- terminal normal,
- log file contains no full media URL/secret,
- session exists only in configured state path.

## 6. Clean-clone rehearsal
Follow README from scratch on a clean checkout/environment. Record every undocumented prerequisite as a release blocker.

## 7. Artifact
Create source archive, list contents, verify forbidden files absent, compute checksum.

## 8. Sign-off
Copy `spec/13_ACCEPTANCE.md` into release QA record and attach evidence/command result for each mandatory checkbox.

Only then tag `v1.0.0`.
