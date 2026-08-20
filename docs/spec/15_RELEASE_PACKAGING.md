# 15 — Release Packaging

## 1. v1.0 release form

The first release is a source release with reproducible documented build/run steps plus Rust release binary built by the user/CI target environment. Do not pretend the Python gateway is a single self-contained Rust binary.

Recommended archive:

```text
ig-reels-cli-v1.0.0-linux-source.tar.gz
```

Optional CI-built Linux binary artifact may accompany it if tested, but still requires Python gateway dependencies and FFmpeg/ffprobe on target system.

## 2. Required release contents

- Rust source + Cargo.toml/Cargo.lock,
- Python gateway source + exact requirements,
- launcher/bootstrap scripts,
- README,
- LICENSE,
- CHANGELOG,
- architecture/security/troubleshooting docs,
- `.env.example` only,
- no runtime state.

## 3. Forbidden release contents

- `.env`,
- `session.json` or any session file,
- media cache,
- logs,
- Python venv,
- Rust `target/`,
- real Reel fixtures,
- screenshots/logs containing private account data,
- credentials,
- signed media URLs.

## 4. Clean-clone release test

On a clean Linux environment/repo clone:
1. install documented system packages,
2. run bootstrap-dev or documented Python venv commands,
3. build Rust release,
4. generate local test clip,
5. run fake gateway/local media checks,
6. configure credentials locally only for manual live test,
7. run live player,
8. quit,
9. inspect cache/log/session locations.

## 5. Versioning

First ship: `v1.0.0`.

Version bump rules:
- patch: bug/security/compatibility without product contract change,
- minor: additive renderer modes/feed sources/options preserving compatibility,
- major: IPC break or fundamental product/architecture break.

IPC protocol version is independent of app semver.

## 6. Release check script

`scripts/release-check.sh` should automate safe checks:
- git clean status,
- tests/format/lint,
- release build,
- forbidden-file pattern scan,
- archive listing scan,
- no `.env`/session/cache/log/target/venv,
- optionally `git grep` secret sentinel patterns.

It must not read or print actual credential values.

## 7. README minimum

README must explain:
- what the app does,
- text-cell video identity,
- screenshot/GIF only if safely produced from generated fixture,
- Linux/system dependencies,
- install/build/run,
- keybindings,
- config paths,
- unofficial Instagram access/account-risk warning,
- troubleshooting auth/challenge,
- troubleshooting terminal truecolor/audio/ffmpeg,
- development/test commands.
