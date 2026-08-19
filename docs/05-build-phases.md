Build Phases (from MANIFESTO.md) with Implementation Details

----------------------------------------------------------------
Phase 1 — Python auth daemon (standalone, testable without Rust)
----------------------------------------------------------------
Files touched:
- py-auth-daemon/requirements.txt
- py-auth-daemon/ig_client.py
- py-auth-daemon/daemon.py
- py-auth-daemon/test_client.py (optional verification tool)
- .env.example (template)
- .gitignore (adds session.json, venv/, etc.)

Verification step:
1. Install dependencies: `cd py-auth-daemon && pip install -r requirements.txt`
2. Set environment variables INSTAGRAM_USERNAME and INSTAGRAM_PASSWORD (copy .env.example to .env and fill, or export).
3. Start daemon: `python daemon.py` – it should print "Daemon listening on /tmp/ig-reels.sock" and "IG client initialized and logged in".
4. In another terminal, run the test client:
   - `python test_client.py ping` → expect {"status": "ok"}
   - `python test_client.py get_reels 3` → expect a map with key "reels" containing up to 3 reel dicts (each with id, video_url, caption, username, like_count). If a challenge is required, response will be {"error":"challenge_required","detail":...}.

Definition of Done (Phase 1):
- daemon.py runs as a Unix domain socket server listening on /tmp/ig-reels.sock.
- It successfully logs into Instagram (using credentials from environment) and persists session to session.json.
- It responds to "ping" with {"status":"ok"}.
- It responds to "get_reels" with a list of reel metadata dictionaries (or appropriate error for challenge/rate limit).
- No crash on malformed requests; errors are returned as msgpack maps.
- The daemon can be stopped and restarted, reusing existing session.json if login is still valid.
- All secrets are sourced from environment, never hardcoded.

----------------------------------------------------------------
Phase 2 — Rust skeleton + IPC (no video yet)
----------------------------------------------------------------
Files touched (to be created):
- rust-tui/Cargo.toml
- rust-tui/src/main.rs
- rust-tui/src/ipc.rs
- rust-tui/src/fetch.rs
- rust-tui/src/app.rs (minimal state machine)
- Possibly rust-tui/src/lib.rs if using library layout (but per manifesto, flat src)

Verification step:
1. Ensure Phase 1 daemon is running.
2. Build and run rust binary: `cd rust-tui && cargo run --release`.
3. The program should:
   - Connect to /tmp/ig-reels.sock.
   - Send a "ping" request via ipc.rs.
   - Print the response (expect "ok") to stdout.
   - Then send a "get_reels" request with count=1 via fetch.rs.
   - Print the received reel metadata as JSON/debug output (no rendering yet).
4. The output must show real reel data fetched from the Python daemon (not mocks).

Definition of Done (Phase 2):
- Rust binary compiles without warnings.
- At startup, it establishes a UDS connection to the daemon.
- It successfully sends a ping request and logs the response.
- It successfully sends a get_reels request (count >=1) and prints the reel metadata (at least one reel) to the terminal.
- No video/audio/download modules are used yet; only IPC and fetch.
- The binary exits cleanly on Ctrl+C or when closed.

----------------------------------------------------------------
Phase 3 — Download + basic playback via mpv (fastest path to "it works")
----------------------------------------------------------------
Files touched (to be created):
- rust-tui/src/download.rs
- rust-tui/src/app.rs (extended to orchestrate download → mpv)
- rust-tui/src/input.rs (basic key handling: n/p/q delegated to mpv or via signals)
- rust-tui/src/render.rs (maybe stub, not used)
- rust-tui/src/decode.rs (stub)
- rust-tui/src/audio.rs (stub)
- mkdir -p rust-tui/tmp-cache (gitignored)

Verification step:
1. Ensure Phase 1 daemon running.
2. Run the Rust binary.
3. The app should:
   - Fetch a batch of reels via IPC.
   - For each reel in order:
        * Call download.rs to fetch MP4 to tmp-cache/<id>.mp4.
        * Spawn a subprocess: `mpv --vo=tct --really-quiet <file>` (or equivalent).
        * Wait for mpv to exit (user presses q inside mpv or end of video).
        * On keypress 'n' or 'p' (if implemented), move to next/previous reel (from local cache, not re-fetch).
   - Basic keybindings: 'q' quits the entire app; 'n'/'p' navigate (may need to kill/respawn mpv).
   - After quit, tmp-cache/ is cleared (or user can configure to keep).

Definition of Done (Phase 3):
- The Rust binary can play Instagram Reels in the terminal using mpv as the video renderer (via libcaca/tct or similar).
- User can advance to next reel with a key (n or →) and go back with p or ← (using locally cached MP4s; no new fetch unless needed).
- User can quit with 'q', which exits the app, kills any child mpv processes, and clears tmp-cache/.
- No custom video rendering or audio synthesis is used; everything is delegated to mpv.
- The app does not crash on end of batch or network hiccups; it shows an error message and continues or exits gracefully.
- All downloaded MP4s are removed upon normal exit.

----------------------------------------------------------------
Phase 4 — Custom terminal rendering (replaces mpv --vo=tct)
----------------------------------------------------------------
Files touched (to be created/replaced):
- rust-tui/src/decode.rs (ffmpeg frame extraction)
- rust-tui/src/render.rs (ANSI truecolor half-block + Kitty protocol)
- rust-tui/src/app.rs (use decode/render instead of mpv subprocess)
- rust-tui/src/input.rs (now handles n/p/space/q internally)
- rust-tui/src/audio.rs (stub for later; may remain stub)
- rust-tui/src/download.rs (unchanged, still prefetches)
- rust-tui/src/ffmpeg-next dependency added to Cargo.toml

Verification step:
1. Ensure Phase 1 daemon running.
2. Run Rust binary.
3. Verify:
   - Video plays inline in terminal (not opening external mpv window).
   - Frame rate is smooth (target fps, e.g., 24‑30).
   - Aspect ratio is preserved (no stretching).
   - Kitty protocol used if terminal supports it (checked via $TERM/kitty detection); fallback to ANSI half‑block.
   - No flicker or tearing beyond acceptable limits (to be iterated).
   - User can pause/resume with space, quit with q, next/previous with n/p/←/→.

Definition of Done (Phase 4):
- Video frames are obtained by extracting raw RGB from MP4 via ffmpeg-next (or ffmpeg CLI) at a steady fps.
- Each frame is converted to terminal pixels:
   * ANSI truecolor using Unicode half‑block (▀) where foreground = top pixel, background = bottom pixel.
   * OR Kitty graphics protocol encoded if $TERM indicates kitty support.
- Rendering is done directly to stdout via crossterm, without spawning external video players.
- Audio is still handled by external mpv (or stub) for this phase—Note: Manifesto says Phase 4 replaces mpv --vo=tct but audio still via mpv? Actually Phase 5 adds audio. So in Phase 4, audio may still be via mpv (we keep mpv for audio only) or we could stub. Per manifesto: Phase 4 is “Custom terminal rendering (replaces mpv --vo=tct)”. It does not mention audio; audio will be added in Phase 5. So in Phase 4 we can still use mpv for audio only (--ao=something --vo=null) or we can start stub. To stay true, we will replace mpv entirely: we will extract audio separately in Phase 5. So for Phase 4 definition of done, we can state that audio is not yet implemented (silent) or we keep using mpv for audio only. However, the manifesto says Phase 3 is “Download + basic playback via mpv (fastest path to 'it works')”. Phase 4 replaces the video output but may still use mpv for audio? It doesn't specify. To be safe, we define that Phase 4 delivers video rendering via custom terminal code; audio may still be handled by mpv (or silent) but the requirement is that video is rendered in-terminal. We'll note that audio is not yet synchronized; that will be Phase 5.
- The app responds to n/p/space/q within the Rust process (no reliance on mpv's keybindings).
- On quit, tmp-cache/ is cleared and terminal restored.

----------------------------------------------------------------
Phase 5 — Audio
----------------------------------------------------------------
Files touched (to be created):
- rust-tui/src/audio.rs (rodio playback, ffmpeg audio extraction)
- rust-tui/src/app.rs (synchronize audio start with video frames)
- rust-tui/src/decode.rs (may need to output timestamps)
- rust-tui/src/render.rs (no change)
- rust-tui/src/Cargo.toml add rodio, possibly rubato for resampling

Verification step:
1. Ensure Phase 1 daemon running.
2. Run Rust binary.
3. Verify:
   - Video plays with synchronized audio (lip‑sync acceptable for short clips; drift noted as limitation).
   - Audio volume is audible, no clipping.
   - Pause/resume (space) pauses both video and audio.
   - Seeking not required; next/previous jumps to start of next reel's audio/video.
   - On end of reel, audio stops cleanly before next reel starts.

Definition of Done (Phase 5):
- Audio track is extracted from the same MP4 via ffmpeg-next (or ffmpeg CLI) decoded to PCM.
- Audio is played via rodio::OutputStream.
- Audio start is aligned with video frame 0 (both start at timestamp 0 from the file).
- Minor drift (<100 ms over 30 seconds) is acceptable for v1; no explicit clock correction needed.
- User can pause/resume with space, which pauses both video and audio streams.
- No audio continues after video ends or when switching reels prematurely.
- All resources (ffmpeg decoders, rodio streams) are properly cleaned up on reel transition and app quit.

----------------------------------------------------------------
Phase 6 — Prefetch + polish
----------------------------------------------------------------
Files touched (to be created/updated):
- rust-tui/src/download.rs (add background prefetch task)
- rust-tui/src/app.rs (manage queue size, trigger prefetch)
- rust-tui/src/ui.rs (status line: username, caption, like count, key hints)
- rust-tui/src/input.rs (ensure n/p/space/q work as described)
- rust-tui/src/render.rs (no change)
- rust-tui/src/decode.rs / audio.rs (no change)
- rust-tui/src/Cargo.toml (no new deps)

Verification step:
1. Ensure Phase 1 daemon running.
2. Run Rust binary and watch a few reels.
3. Verify:
   - While reel N is playing, download of reel N+1+ (at least one ahead) occurs in background (observable via network traffic or tmp-cache timestamps).
   - Switching to next reel incurs no noticeable stall (if prefetch succeeded).
   - Status line shows current reel's username, truncated caption (if any), like count, and keybind hints (n/p/space/q).
   - Keybindings: n/j/→ = next, p/k/← = previous, space = pause/resume, q = quit.
   - On quit: alt‑screen cleared, terminal mode restored, tmp-cache/ emptied, child processes killed.
   - No leftover tmp-cache files after normal exit.

Definition of Done (Phase 6):
- Prefetch logic ensures at least one reel (N+1) is fully downloaded before reel N ends, minimizing stall.
- UI status line is rendered continuously above/below video (or overlay) without interfering with frame timing.
- All keybindings work as described and are responsive (<50 ms latency).
- Graceful cleanup on SIGINT (Ctrl+C) or 'q': restore terminal, kill child ffmpeg/rodio processes, remove all files in tmp-cache/.
- The app runs indefinitely (or until user quits) without memory leaks or unbounded growth in tmp-cache/ (files cleared after each reel or on exit).
- No hardcoded credentials; all configuration via environment/.env.

----------------------------------------------------------------
After Phase 6, the application meets the Definition of Done for v1 (see MANIFESTO.md Section 7).