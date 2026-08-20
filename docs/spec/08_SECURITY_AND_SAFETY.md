# 08 — Security and Safety

## 1. Threat model

This is a local personal app but processes:
- credentials/session material,
- untrusted remote strings,
- signed remote URLs,
- untrusted media files,
- terminal control surface,
- filesystem cleanup paths,
- external process outputs.

Treat each as a trust boundary.

## 2. Credential boundary

Only Python gateway receives:
- `IG_USERNAME`,
- `IG_PASSWORD`,
- session file content.

Rust must not require or log these values. Launcher removes credential env vars before launching Rust where practical.

No password CLI flag.

## 3. Session file

Outside repo. Mode `0600` where supported. Parent state dir private enough for user-only app state.

Do not print/dump entire instagrapi settings.

## 4. Signed media URL

Treat as sensitive transient data:
- SecretUrl/RedactedUrl wrapper in Rust,
- Debug -> `<redacted-url>` or host-only safe form,
- never normal logs,
- never panic context,
- never task snapshots/fixtures,
- no query-string logging.

Downloader still receives actual URL internally.

## 5. HTTPS redirects

Initial URL and every redirect target must use `https`.

Max five hops. Custom redirect policy must itself enforce limit because custom policies may not inherit default loop protection.

Do not enable certificate validation bypass.

## 6. HTTP environment proxies

Reqwest may honor system proxy environment by default depending build/config. v1.0 does not implement proxy rotation or proxy management. Document actual behavior chosen in dependency task. Do not add stealth proxy logic.

## 7. Remote text sanitizer

Input: username/caption/error detail from upstream.

Output status string rules:
- remove ESC (`U+001B`),
- remove C0 controls `U+0000..U+001F` except convert `\t`, `\n`, `\r` to single spaces,
- remove DEL `U+007F`,
- remove C1 `U+0080..U+009F`,
- collapse runs of whitespace to one space,
- trim,
- truncate to UI budget,
- never interpret as format/control sequence.

Unit tests include embedded `\x1b[2J`, OSC-like text, newlines, tabs, NUL.

## 8. Media process safety

Never build shell strings.

Pass local file path as a single argument to ffmpeg/ffprobe/mpv.

Stdin null to prevent media child from consuming terminal input.

Bound captured stderr.

## 9. Cache path safety

- run directory is direct child of validated cache root,
- filename from safe encoded Reel id/hash,
- no remote URL path components,
- stale cleanup refuses symlinks where distinguishable,
- recursive delete only on validated owned run child,
- never delete `/`, `$HOME`, cache root, repo, or arbitrary env-provided path.

## 10. Terminal safety

- remote text never enters ANSI builder except as sanitized bytes after reset,
- display owner is sole stdout writer,
- alternate screen/raw mode guarded by RAII/best effort,
- disable line wrap only with matching restore,
- hide cursor only with matching show,
- panic hook attempts restore before diagnostic.

## 11. Account-risk boundary

Instagram access is unofficial/private API based. v1.0 intentionally avoids:
- bulk scraping,
- engagement automation,
- challenge automation,
- anti-detection behavior,
- proxy rotation,
- rate-limit bypass.

If upstream restricts access, surface it; do not make the app stealthier as a maintenance fix.

## 12. Release secret scan

Before ship:
- `git grep` for username/password test values,
- inspect git history if real credentials were ever accidentally added,
- inspect release archive listing,
- ensure `.env`, session, media cache, logs excluded,
- use sentinel secret tests in logger/sanitizer where applicable.
