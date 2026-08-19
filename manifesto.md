# IG-REELS-CLI — BUILD MANIFESTO

You are implementing a terminal application that lets a user scroll and watch
Instagram Reels inside their terminal, using keyboard controls, with video and
audio playing inline. This document is the single source of truth for scope,
architecture, and sequencing. Read it fully before writing any code. Do not
deviate from the architecture without flagging the deviation explicitly.

---

## 0. NON-NEGOTIABLE RULES

1. **Never touch the Python auth/daemon code without being told to.** That
   layer talks to Instagram's private API. Getting it wrong risks the user's
   account getting flagged or banned. If a task involves login, session,
   challenge/2FA handling, or rate limiting — STOP and ask for explicit
   confirmation before writing or changing anything there.
2. **One function or module per task.** Do not generate multi-file, multi-hundred-line
   changes in one pass. Small unit → compile/run → confirm → next unit.
3. **Never invent an API that doesn't exist.** If you're not sure whether a
   crate/library function exists with that signature, say so instead of
   guessing. Hallucinated APIs are the #1 failure mode to avoid here.
4. **No secrets in code.** Credentials come from `.env`, never hardcoded,
   never logged, never printed to stdout/stderr.
5. **Every external process boundary (Rust ↔ Python, Rust ↔ ffmpeg, Rust ↔
   mpv) must have explicit error handling.** A silent failure across a
   process boundary is the hardest bug class in this project — don't create
   more of them than necessary.
6. **Ask before deleting or overwriting working code.** If something already
   works, don't "clean it up" as a side effect of an unrelated task.

---

## 1. WHAT WE'RE BUILDING

A local, single-user, terminal-only application:

- Logs into Instagram (once), persists the session.
- Fetches a batch of Reels (explore/hashtag feed).
- Downloads each reel's video to local temp storage, one ahead of playback
  (prefetch).
- Plays the video inline in the terminal (video + audio), full screen or near
  full screen in the terminal viewport.
- Lets the user press keys to go to next/previous reel, pause, quit.
- Shows a thin status line (username/caption/like count if available).

No web UI. No database. No multi-user concerns. No posting/liking/commenting
automation in v1 — this is read-only playback.

---

## 2. ARCHITECTURE OVERVIEW

Two processes, one IPC boundary:

```
┌─────────────────────────┐        UDS + msgpack        ┌──────────────────────────┐
│   Python auth daemon      │ <──────────────────────────> │      Rust TUI app        │
│   (instagrapi)          │        request/response      │  (everything else)       │
└─────────────────────────┘                              └──────────────────────────┘
```

- **Python daemon**: does exactly three things — login, refresh session,
  return a batch of reel metadata + video URLs. Nothing else. It is a thin
  translation layer over instagrapi, not an application.
- **Rust app**: owns the terminal, the render loop, video decode, audio
  playback, keypress handling, prefetch/caching, and orchestration. This is
  where 90% of the code and 90% of the polish work lives.

They communicate over a **Unix domain socket** using **msgpack**-encoded
request/response messages. Not PyO3 — we are not embedding the Python
interpreter in the Rust binary. Keep them as separate OS processes so either
side can crash/restart independently.

---

## 3. REPO LAYOUT

```
ig-reels-cli/
├── README.md
├── .env.example
├── .gitignore              # must include: .env, session.json, /py-auth-daemon/venv, /target, /tmp-cache
├── py-auth-daemon/
│   ├── daemon.py            # entrypoint, UDS server loop
│   ├── ig_client.py           # thin instagrapi wrapper
│   ├── requirements.txt
│   └── session.json          # gitignored, generated at runtime
└── rust-tui/
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs            # entrypoint, sets up tokio runtime, wires modules together
    │   ├── ipc.rs            # UDS client, msgpack request/response types
    │   ├── fetch.rs            # calls ipc to get reel batches, manages the "queue" of upcoming reels
    │   ├── download.rs          # downloads reel mp4s to tmp-cache, prefetch logic
    │   ├── decode.rs            # ffmpeg frame extraction
    │   ├── render.rs            # ANSI truecolor / kitty protocol terminal blitting
    │   ├── audio.rs            # rodio playback, sync with video frames
    │   ├── input.rs            # crossterm raw-mode keypress capture
    │   ├── ui.rs              # status line, minimal chrome
    │   └── app.rs              # top-level state machine tying it all together
    └── tmp-cache/             # gitignored, downloaded videos live here transiently
```

---

## 4. BUILD ORDER (DO NOT SKIP AHEAD)

Each phase must fully work and be manually verified before starting the
next. This is the actual sequencing — follow it in order.

### Phase 1 — Python auth daemon (standalone, testable without Rust)
1. `ig_client.py`: wrap instagrapi. Functions: `login(username, password)`,
   `load_or_login()` (tries session.json first, falls back to fresh login),
   `get_reels_batch(count)` → returns list of `{id, video_url, caption,
   username, like_count}`.
2. `daemon.py`: opens a UDS socket at a fixed path (e.g.
   `/tmp/ig-reels.sock`), listens for msgpack requests `{"cmd": "get_reels",
   "count": N}` and `{"cmd": "ping"}`, responds with msgpack.
3. **Verify standalone**: use `socat` or a tiny Python test client to hit the
   socket manually and confirm you get real reel data back. Do this before
   any Rust code exists.
4. Handle the login challenge path explicitly: if instagrapi raises a
   challenge/checkpoint exception, the daemon should return a clear error
   response (`{"error": "challenge_required", "detail": ...}"`) rather than
   hanging or crashing. Actually solving the challenge is a manual,
   human-in-the-loop step the first time — don't try to automate it away.

### Phase 2 — Rust skeleton + IPC (no video yet)
1. `main.rs` + `ipc.rs`: connect to the UDS socket, send a `ping`, print the
   response. This confirms the boundary works.
2. `fetch.rs`: call `get_reels`, print the returned metadata as JSON/debug
   output to the terminal. No rendering yet.
3. **Verify**: running the Rust binary prints real reel metadata fetched
   live through the Python daemon.

### Phase 3 — Download + basic playback via mpv (fastest path to "it works")
1. `download.rs`: given a video URL, download to `tmp-cache/<id>.mp4`.
2. Shell out to `mpv --vo=tct <file>` as a subprocess, blocking until it
   exits or the user quits it.
3. `input.rs` + `app.rs`: simple loop — fetch batch → for each reel: download
   → play via mpv → wait for exit → next. Basic n/p/q handled by mpv itself
   for now (mpv has its own keybinds) or by killing/respawning the subprocess
   on keypress.
4. **This phase alone is a legitimate, demoable v0.** Stop here and confirm
   end-to-end works before touching custom rendering.

### Phase 4 — Custom terminal rendering (replaces mpv --vo=tct)
1. `decode.rs`: use `ffmpeg-next` (or shell out to `ffmpeg` CLI producing
   raw frames) to extract frames from the downloaded mp4 as RGB buffers at a
   target frame rate.
2. `render.rs`: convert each RGB frame buffer into terminal output.
   - Start with **ANSI truecolor half-block rendering** (▀ character, fg =
     top pixel color, bg = bottom pixel color) — this works everywhere.
   - Add **kitty graphics protocol** support behind a terminal-capability
     check (detect `$TERM` / kitty-specific env vars) as a higher-fidelity
     path for supporting terminals.
3. **Verify**: play a short test clip and confirm frame output isn't
   flickering, aspect ratio is correct, and it resizes reasonably when the
   terminal window resizes.

### Phase 5 — Audio
1. `audio.rs`: extract audio track via ffmpeg, play via `rodio`.
2. Sync audio start to video start; accept minor drift for v1 rather than
   building a full AV-sync clock — note this as a known limitation, not a
   silent bug.

### Phase 6 — Prefetch + polish
1. While reel N plays, `download.rs` should already be fetching reel N+1 in
   the background (tokio task) so there's no stall on "next."
2. `ui.rs`: thin status line above/below video — username, caption
   (truncated), like count, keybind hints.
3. Keybinds: `n`/`j`/`→` = next, `p`/`k`/`←` = previous (from local
   history/cache, not re-fetch), `space` = pause/resume, `q` = quit and
   clean up tmp-cache.
4. Graceful shutdown: on quit, kill any child processes, clear alt-screen,
   restore terminal mode, delete tmp-cache contents.

---

## 5. KEY TECHNICAL DECISIONS (DO NOT RELITIGATE THESE)

| Concern | Decision | Why |
|---|---|---|
| IG API access | instagrapi (Python), unofficial | Only mature maintained option |
| Rust↔Python bridge | UDS socket + msgpack | Process isolation, no embedded interpreter fragility |
| Video decode | ffmpeg (via `ffmpeg-next` or CLI) | Industry standard, no reason to reinvent |
| Terminal video render | Hand-rolled ANSI truecolor + optional kitty protocol | Full control, avoids TUI-framework/video-subprocess conflicts |
| TUI framework for chrome | None required, or minimal `crossterm` direct use | ratatui's widget model fights a per-frame video blit loop |
| Audio | `rodio` | Pure Rust, no runtime binary dependency |
| Async runtime | `tokio` | Needed for concurrent prefetch + input + IPC |

If the agent (or you) find a reason one of these needs to change mid-build,
that's a real architecture decision — pause and think it through explicitly,
don't silently swap libraries.

---

## 6. THINGS THAT WILL GO WRONG (EXPECTED, NOT BUGS TO PANIC ABOUT)

- **Instagram challenge/checkpoint on first login**, or after IP change. Human
  must solve this once manually. Session persistence (`session.json`) avoids
  repeat triggers as long as the daemon reuses it.
- **instagrapi endpoints break** when Instagram changes their private API.
  This is normal for unofficial clients — check for library updates if fetch
  calls start failing.
- **Terminal flicker/tearing** during rendering — expect to iterate on this
  visually, it's not a one-shot fix.
- **AV drift** over longer clips without a real sync clock — acceptable for
  v1, flag as future work, don't over-engineer it now.

---

## 7. DEFINITION OF DONE (v1)

- `daemon.py` runs standalone, logs in once, persists session, serves reel
  batches over the socket.
- Rust binary: connects to daemon, fetches a batch, downloads reels one
  ahead, renders video+audio inline in terminal using hand-rolled ANSI
  rendering, responds to n/p/space/q, cleans up on exit.
- No crashes on normal quit. No leftover temp files after quit. No
  credentials anywhere in git history.

Anything beyond this (posting, liking, following, stories, DMs) is out of
scope unless explicitly requested as a v2.