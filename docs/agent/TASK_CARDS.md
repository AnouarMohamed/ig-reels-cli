# Implementation Task Cards — v1.0 From Empty Repo to Ship

**Rule:** give the coding agent exactly one card at a time. Every card also obeys `AGENT_RULES.md` and applicable commands in `COMMANDS.md`.

---

# Phase 0 — Repository and dependency boundary

## T-001 — Create safe repository skeleton
**Read:** Manifesto; `spec/12_REPO_LAYOUT.md`, `spec/08_SECURITY_AND_SAFETY.md`.
**Allowed:** top-level directories, `.gitignore`, `.env.example`, minimal `README.md` placeholder only.
**Goal:** create exact target directories with no behavior.
**Required:** ignore `.env`, sessions, `target/`, venv, Python caches, logs, cache/run directories.
**Gate:** `git status --ignored` demonstrates patterns; `git ls-files` contains no runtime file.
**Forbidden:** Cargo/Python dependencies, app code.

## T-002 — Pin Python gateway dependencies
**Read:** `spec/14_CONFIGURATION_AND_DEPENDENCIES.md`.
**Allowed:** `py-ig-gateway/requirements.txt`, optional test requirements file if project policy chooses one.
**Goal:** exact runtime pins for `instagrapi==2.18.16`, `msgpack==1.2.1`; add only minimal test dependency needed by chosen test runner.
**Gate:** clean venv install succeeds; record installed versions.
**Forbidden:** gateway code.

## T-003 — Create Rust crate and locked baseline
**Read:** `spec/14_CONFIGURATION_AND_DEPENDENCIES.md`, `spec/12_REPO_LAYOUT.md`.
**Allowed:** `rust-tui/Cargo.toml`, `Cargo.lock`, minimal `src/main.rs` printing a non-secret placeholder before any TUI.
**Goal:** create compileable crate with only dependencies needed by near-term tasks; do not add all future crates blindly if unused warnings/features complicate build.
**Required:** commit Cargo.lock.
**Gate:** `cargo check --locked`, fmt.

## T-004 — Shared configuration/path parser skeleton
**Read:** `spec/01_ARCHITECTURE.md`, `spec/14_CONFIGURATION_AND_DEPENDENCIES.md`.
**Allowed:** `rust-tui/src/config.rs`, minimal main wiring, tests.
**Goal:** parse shared path env vars and renderer constants; validate numeric bounds; no Instagram credentials.
**Gate:** tests for valid/default/invalid FPS, max cols, max rows, fallback aspect.
**Forbidden:** filesystem creation, logging, TUI.

---

# Phase 1 — IPC with no Instagram

## T-005 — Python framed MessagePack codec
**Read:** `spec/02_IPC_PROTOCOL.md`.
**Allowed:** `py-ig-gateway/protocol.py`, tests.
**Goal:** pure frame encode/decode helpers, no sockets.
**Required tests:** zero/oversize/truncated/invalid payload, raw=False string behavior, protocol map shape helper if used.
**Forbidden:** instagrapi.

## T-006 — Python fake gateway
**Read:** IPC spec, test strategy.
**Allowed:** `scripts/fake-gateway.py`, supporting fixture data.
**Goal:** UDS server with auth-free `ping` and deterministic `get_reels`; one request/response per connection.
**Required:** fake URL values use harmless example HTTPS URLs; selectable error response mode.
**Gate:** tiny Python client manually pings and gets 2+ fake DTOs.

## T-007 — Rust IPC DTO types
**Read:** IPC spec.
**Allowed:** `rust-tui/src/ipc.rs`, Cargo dependency changes strictly needed for serde/rmp-serde, unit tests.
**Goal:** request/response/error/Reel DTO structures only.
**Required:** serde named fields; validation helpers for DTO limits.
**Forbidden:** socket connect.

## T-008 — Rust IPC frame reader/writer
**Read:** IPC spec; ADR-0001.
**Allowed:** `ipc.rs`, tests.
**Goal:** async 4-byte big-endian framing with 1 MiB cap and exact reads.
**Tests:** partial header/payload, zero, oversize, truncated.

## T-009 — Rust UDS request client
**Read:** IPC spec, architecture.
**Allowed:** `ipc.rs`, focused integration test/main diagnostic.
**Goal:** timeout-bounded connect -> one request -> one response -> close; request_id validation.
**Gate:** ping fake gateway.
**Forbidden:** feed model.

## T-010 — Cross-language MessagePack contract
**Read:** IPC spec.
**Allowed:** Python/Rust protocol tests, `scripts/test-protocol-contract.sh`, literal fixture files.
**Goal:** prove Python bytes decode in Rust and Rust named-map bytes decode in Python.
**Required:** test fails if Rust uses compact tuple struct encoding.
**Gate:** script exits 0.

---

# Phase 2 — Real protected Instagram gateway

## T-011 — Python gateway configuration
**Protected gateway task.**
**Read:** security/config specs.
**Allowed:** `py-ig-gateway/config.py`, tests.
**Goal:** validate socket/session/log paths and credential presence without logging values; create private parent dirs where appropriate.
**Forbidden:** instagrapi client/login.

## T-012 — Pure ReelDTO normalization adapter
**Protected gateway task.**
**Read:** IPC spec; inspect pinned instagrapi model/source.
**Allowed:** `ig_client.py`, mapping tests.
**Goal:** convert mocked Media-like object to exact DTO fields.
**Tests:** null caption/likes, missing required URL/user/id, numeric conversion, no upstream object serialization leakage.
**Forbidden:** network/login.

## T-013 — Lazy session reuse/login initialization
**Protected gateway task.**
**Read:** security/config; current pinned instagrapi session docs/source.
**Allowed:** `config.py`, `ig_client.py`, focused tests.
**Goal:** one guarded `ensure_authenticated` attempt per process: load persisted settings first, fresh login only when required, persist refreshed settings.
**Required:** session file permissions best effort `0600`.
**Forbidden:** automatic challenge solving, retry loop, account stealth.

## T-014 — Reels/Discover fetch adapter
**Protected gateway task.**
**Read:** product scope, IPC error codes; inspect pinned instagrapi API immediately before coding.
**Allowed:** `ig_client.py`, tests with mocked upstream.
**Goal:** one read-only method returning normalized DTO list.
**Required:** stable error classification; max requested count enforcement.
**Forbidden:** hashtag/following fallback or scraping workaround.

## T-015 — Real gateway daemon
**Protected gateway task.**
**Read:** architecture socket lifecycle, IPC, errors/security.
**Allowed:** `daemon.py`, daemon tests, minimal safe file logging setup.
**Goal:** bind socket, auth-free ping, serial request dispatch, lazy auth on get_reels, framed responses.
**Required:** healthy existing socket -> already_running; stale unlink only after validation.
**Gate:** ping with no credentials/network auth.

## T-016 — Live gateway/session manual gate
**Protected gateway task; ideally no source edits.**
**Read:** risk register.
**Allowed:** only tiny diagnostic script if absolutely required by card; no architecture changes.
**Goal:** prove real batch and ordinary session reuse.
**Manual steps:** run gateway, get redacted DTO metadata; restart; confirm persisted session path reused; trigger/observe auth error handling if available without risking account.
**PASS evidence:** Reel ids/usernames/count only; never paste URL/session/password.
**Stop:** if pinned instagrapi path unavailable -> ADR/evidence, no fallback coding.
**Freeze:** after PASS, Python gateway is frozen unless later explicit protected task.

---

# Phase 3 — Rust domain, logging, feed, cache, download

## T-017 — Rust domain model and redacted URL wrapper
**Read:** security, feed/cache.
**Allowed:** `model.rs`, tests.
**Goal:** `ReelMeta`, `CachedReel`, `SecretUrl/RedactedUrl` with Debug that cannot reveal full URL.
**Tests:** debug/display sentinel URL absent.

## T-018 — Remote text sanitizer
**Read:** security sanitizer rules.
**Allowed:** `ui.rs` or dedicated sanitizer module if layout allows; tests.
**Goal:** remove ESC/C0/C1/DEL, normalize whitespace, trim/truncate helper.
**Required tests:** `\x1b[2J`, OSC-like string, NUL, tabs/newlines.

## T-019 — Structured file logging foundation
**Read:** observability/security/config.
**Allowed:** `logging.rs`, minimal Cargo/config/main wiring, tests.
**Goal:** tracing to file under log dir; no TUI stdout.
**Gate:** secret and signed-URL sentinel absent from log.

## T-020 — Pure feed queue/dedup
**Read:** `spec/03_FEED_QUEUE_CACHE.md`.
**Allowed:** `feed.rs`, tests.
**Goal:** unseen queue, seen_ids, low-water/target calculations independent of network.
**Tests:** duplicates in/across batch, empty batch, current/history ids excluded.

## T-021 — Pure back/forward navigation model
**Read:** feed/cache, state spec.
**Allowed:** `feed.rs` and tests only.
**Goal:** deterministic back/current/forward transitions and five-item logical bound; nearest item is always deque back; farthest is front.
**Tests:** A->B->C, Previous twice, Next twice; assert exact deque contents after each step; new branch clears forward; when total >5 evict front from larger side (back on tie).

## T-022 — Safe run-cache path primitives
**Read:** cache/security.
**Allowed:** `cache.rs`, tests.
**Goal:** validate cache root, create `run-<pid>-<random>`, safe Reel filename, safe final/part paths.
**Forbidden:** HTTP.

## T-023 — Stale run cleanup
**Read:** cache/security.
**Allowed:** `cache.rs`, tests.
**Goal:** identify/remove only validated old direct-child `run-*` dirs; no symlink/root escape.
**Tests:** malicious names, nested path, symlink where platform test supports, cache root preservation.

## T-024 — Reqwest downloader client policy
**Read:** download/security/dependency references.
**Allowed:** `download.rs`, Cargo changes for reqwest, pure/policy tests.
**Goal:** reusable client with 10s connect/60s timeout, no cookie jar, max-5 custom redirect, HTTPS every hop.
**Required:** verify pinned reqwest redirect API.
**Forbidden:** file writing.

## T-025 — Atomic streaming download
**Read:** feed/cache/security/errors.
**Allowed:** `download.rs`, local integration tests.
**Goal:** stream to unique `.part`, enforce 150 MiB from header and actual bytes, rename on success only, remove part on failure/cancel.
**Gate:** local server fixed/chunked/oversize/disconnect cases.

## T-026 — Real Reel download gate
**Read:** risk register.
**Allowed:** minimal diagnostic wiring; do not modify gateway.
**Goal:** fetch one real DTO through IPC and download using Rust with no Instagram cookie/session headers.
**PASS output:** Reel id, byte count, local path only.
**Stop:** if cookies/session required -> ADR.

---

# Phase 4 — Media smoke, fixtures, probe, geometry

## T-027 — mpv tct end-to-end smoke
**Read:** ADR-0002; build plan.
**Allowed:** isolated smoke module/script.
**Goal:** spawn `mpv --vo=tct <downloaded-file>` with arg vector and check exit status.
**Forbidden:** final navigation/TUI integration.
**Manual gate:** real downloaded Reel visibly plays in terminal via mpv.

## T-028 — Deterministic AV fixture generator
**Read:** test strategy.
**Allowed:** `scripts/make-test-clips.sh`, fixture README.
**Goal:** generate landscape >=20s MP4/AAC and portrait MP4/AAC with color/motion/audio.
**Required:** deterministic names under ignored/generated test directory; ffprobe verification.

## T-029 — FFmpeg/ffprobe runtime preflight
**Read:** config/dependencies/errors.
**Allowed:** `media_probe.rs` or preflight module per layout, config/error wiring, tests with fake PATH where feasible.
**Goal:** spawn `ffmpeg -version`, `ffprobe -version`; clear fatal errors if missing.
**Forbidden:** parse media yet.

## T-030 — ffprobe JSON process wrapper
**Read:** media-probe spec; ADR-0002.
**Allowed:** `media_probe.rs`, tests.
**Goal:** bounded argv spawn/capture/timeout; parse JSON into intermediate structures.
**Required:** no human stderr parsing; stderr capped.

## T-031 — Normalize MediaInfo
**Read:** media-probe spec.
**Allowed:** `media_probe.rs`, tests.
**Goal:** width/height/DAR/audio/duration normalization with fallbacks.
**Tests:** missing DAR, SAR fallback, invalid zero/nonfinite, no audio.

## T-032 — Rotation/orientation proof
**Read:** media-probe geometry spec, risk notes.
**Allowed:** test fixture script, `media_probe.rs` only if normalization fix needed.
**Goal:** generate/obtain rotated metadata fixture and prove MediaInfo DAR matches FFmpeg decoded orientation.
**Gate:** documented automated/manual result.
**Stop:** if mismatch cannot be represented by current MediaInfo -> fix within spec, not renderer stretching.

## T-033 — Pure terminal cell-ratio calculation
**Read:** geometry spec; current crossterm window_size docs.
**Allowed:** `geometry.rs`, tests.
**Goal:** measured ratio when valid; fallback 2.0; clamp/validation.
**Tests:** zero pixel metrics, absurd ratios, valid metrics.

## T-034 — Pure render-geometry calculator
**Read:** geometry ADR/spec.
**Allowed:** `geometry.rs`, tests.
**Goal:** implement normative equation and largest-fitting integer rectangle; even sample height; centered origin.
**Gate:** golden cases including 9:16 120×40; <=3% DAR error normal cases.
**Forbidden:** ffmpeg or terminal I/O.

---

# Phase 5 — Raw decoder and text renderer

## T-035 — FFmpeg decode argv builder
**Read:** playback spec, geometry, ADR-0002.
**Allowed:** `decode.rs`, tests.
**Goal:** build executable+args for local file -> fixed W×H, 15 FPS, RGB24 rawvideo stdout, no audio.
**Required:** verify exact installed/pinned-compatible FFmpeg syntax with generated fixture.
**Forbidden:** spawning yet if clean separation allows.

## T-036 — FFmpeg child spawn/stderr drainer
**Read:** playback/errors/security.
**Allowed:** `decode.rs`, tests.
**Goal:** spawn with stdin null/stdout+stderr piped; bounded stderr tail <=64 KiB; no shell.
**Gate:** generated fixture child starts and stderr cannot block.

## T-037 — Whole RGB frame reader
**Read:** playback/renderer.
**Allowed:** `decode.rs`, tests.
**Goal:** checked `W*H*3`, exact frame reads, frame_index, clean EOF vs partial-frame error.
**Tests:** synthetic reader exact two frames, partial final frame.

## T-038 — Bounded decoder worker + cancellation
**Read:** playback/concurrency.
**Allowed:** `decode.rs`, tests.
**Goal:** capacity-3 channel, cancellation token, child kill/wait, reader/drainer termination.
**Gate:** long fixture cancel returns boundedly and leaves no child.

## T-039 — Renderer core types and frame validation
**Read:** renderer spec.
**Allowed:** `renderer.rs`, tests.
**Goal:** RGB type/frame/geometry/render-frame types; validate even height/exact length/checked arithmetic.
**Forbidden:** ANSI or stdout.

## T-040 — One half-block cell encoder
**Read:** ADR-0004, renderer spec.
**Allowed:** `renderer.rs`, tests.
**Goal:** pure `(top RGB,bottom RGB)->TerminalCell` or equivalent semantic output.
**Gate:** red/blue and green/white tiny test vectors.

## T-041 — ANSI truecolor encoder primitives
**Read:** renderer spec; verified xterm reference.
**Allowed:** `ansi.rs`, tests.
**Goal:** append FG 38;2, BG 48;2, reset, cursor-move commands to byte buffer with no format-string injection.
**Required:** prefer crossterm commands where they cleanly support buffering; otherwise byte helpers must be exact/tested.

## T-042 — ANSI color-state coalescing
**Read:** renderer/performance.
**Allowed:** `ansi.rs`, tests.
**Goal:** avoid repeating FG/BG command when unchanged; still exact colors.
**Tests:** uniform row command counts/semantics.
**Forbidden:** quantization/diffing.

## T-043 — Full RGB frame -> ANSI video buffer
**Read:** renderer spec.
**Allowed:** `renderer.rs`, `ansi.rs`, tests.
**Goal:** explicit cursor move per row, encode every cell, no newline/wrap dependence, reset at end.
**Gate:** glyph count equals W*H/2; tiny golden semantic tests.

## T-044 — Status-line encoder integration
**Read:** renderer + sanitizer/UI specs.
**Allowed:** `ui.rs`, renderer/display interface tests.
**Goal:** trusted static hints + sanitized remote text only after reset; one row; truncation.
**Gate:** injected ESC cannot appear as active control from remote field.

## T-045 — Local visual text-video renderer smoke
**Read:** renderer acceptance/performance.
**Allowed:** a dedicated local-render diagnostic binary/module/script, renderer only if a proven bug is found.
**Goal:** generated portrait/landscape fixtures -> ffprobe -> geometry -> ffmpeg -> renderer -> terminal at 15 FPS, no Instagram/audio/navigation.
**Manual gate:** recognizable motion/colors, correct portrait aspect, no scroll, cursor hidden/restored.
**Stop:** do not add image protocol if quality/perf disappoints.

---

# Phase 6 — Audio and scheduling

## T-046 — Rodio generated-fixture audio spike
**Read:** playback/audio ADR, dependencies, risk register.
**Allowed:** `audio.rs`, Cargo feature changes only as required, focused diagnostic/test.
**Goal:** prove pinned current rodio can decode generated MP4/AAC, pause/play, and report position.
**Gate:** manual audio output + position behavior recorded.
**Stop:** if API differs, verify and adjust within pin; if media support absent, ADR.

## T-047 — Rodio real-Reel audio spike
**Read:** risk register.
**Allowed:** diagnostic only; no broad integration.
**Goal:** same proof on one real downloaded Reel.
**Gate:** audible playback and advancing position or explicit supported no-audio Reel handling.
**Stop:** real Reel decode failure -> ADR before extraction workaround.

## T-048 — Audio service/capability state
**Read:** playback/state/errors.
**Allowed:** `audio.rs`, tests.
**Goal:** `Unknown/Available/Unavailable`, prepare paused player/source, per-Reel no-audio/decode failure result.
**Forbidden:** scheduler.

## T-049 — Video-only monotonic scheduler
**Read:** playback spec.
**Allowed:** `scheduler.rs`, tests with fake time if practical.
**Goal:** frame target from monotonic position, drop late frames, pause/resume accounting.
**Gate:** deterministic synthetic frames; no unbounded queue.

## T-050 — Audio-master scheduler
**Read:** ADR-0007, playback.
**Allowed:** `scheduler.rs`, audio interface tests.
**Goal:** target_index=floor(audio_pos*FPS), stale frame dropping, no audio stall.
**Tests:** scripted fake audio positions and decoded frame sequences.

## T-051 — Playback generation controller
**Read:** playback architecture/state.
**Allowed:** a focused playback/controller module if layout chooses, `decode.rs`, `audio.rs`, `scheduler.rs` interfaces only as necessary.
**Goal:** own probe/geometry/decoder/audio/scheduler lifecycle under one generation/cancellation token; emit typed events/render frames.
**Forbidden:** navigation state and stdout.

## T-052 — Local AV playback smoke
**Read:** playback acceptance.
**Allowed:** diagnostic/integration wiring.
**Goal:** generated >=20s clip with text video + audio master, pause 3s/resume, clean end.
**Manual gate:** no obvious accumulating drift; pause does not create permanent offset; child cleanup passes.

---

# Phase 7 — Terminal ownership and App state

## T-053 — TerminalGuard
**Read:** architecture/security; crossterm docs.
**Allowed:** `terminal.rs`, tests/demo.
**Goal:** enter alternate screen, raw mode, hide cursor, disable wrap if used; Drop/restore inverse operations safely.
**Gate:** manual enter/exit leaves shell normal.

## T-054 — Terminal metric and capability query
**Read:** geometry/dependencies; `spec/17_TERMINAL_COMPATIBILITY.md`.
**Allowed:** `terminal.rs`, tests where possible.
**Goal:** return cols/rows + optional pixel metrics without deciding geometry; classify `TERM=dumb` as unsupported and truecolor advertisement as Likely/Unknown without a giant terminal allowlist; zero pixel metrics allowed.
**Gate:** tests for TERM=dumb, COLORTERM=truecolor/24bit, unknown-but-usable classification.

## T-055 — Cancellable input worker
**Read:** state/concurrency; crossterm event docs.
**Allowed:** `input.rs`, tests/demo.
**Goal:** blocking worker with ~50ms `poll`, then `read`; map only required keys/resize; cancellation checked between polls.
**Forbidden:** direct App mutation.

## T-056 — DisplayOwner
**Read:** architecture/renderer; `spec/18_INTERNAL_EVENT_AND_DISPLAY_CONTRACTS.md`.
**Allowed:** `display.rs`, tests.
**Goal:** sole TUI stdout writer with reliable bounded control queue plus latest-value frame slot; control has priority; stale generation frames dropped.
**Required:** `BeginGeneration`, generation-scoped/global Status, SuspendTooSmall, Shutdown; BeginGeneration clears stale viewport once; video frames never accumulate in an unbounded queue.
**Tests:** publish old frame then BeginGeneration(new) and prove old frame cannot write afterward; flood latest-frame slot and prove only newest values need be presented; Shutdown/control not starved.

## T-057 — Pure App state enum/data/events
**Read:** state spec.
**Allowed:** `app.rs`, tests.
**Goal:** types and pure reducer-like transition logic without spawning workers.

## T-058 — App startup transitions
**Read:** state/build plan.
**Allowed:** `app.rs`, tests.
**Goal:** ConnectingGateway -> Preflight -> LoadingFeed -> Buffering -> Playing; fatal startup paths.

## T-059 — App navigation/pause transitions
**Read:** state/feed.
**Allowed:** `app.rs`, tests.
**Goal:** Next/Previous/Space/natural end, Switching key coalescing policy, history semantics.

## T-060 — App stale-op/generation handling
**Read:** state/concurrency.
**Allowed:** `app.rs`, tests.
**Goal:** stale feed/download/probe/playback results ignored deterministically.

## T-061 — Shutdown coordinator
**Read:** state/architecture/security.
**Allowed:** `shutdown.rs`, minimal app/terminal interfaces, tests.
**Goal:** global cancel, task wait, playback kill, terminal restore ordering, cache cleanup, log flush.
**Gate:** local fixture quit and Ctrl-C no child/cache/terminal damage.

---

# Phase 8 — Integrate local app, then live feed

## T-062 — Local fixture full-app integration
**Read:** all media/state specs relevant.
**Allowed:** `main.rs`, app orchestration interfaces, integration test/diagnostic; no gateway changes.
**Goal:** run full TUI using local generated playlist only: play/pause/next/previous/quit.
**Gate:** manual local fixture navigation + cleanup.

## T-063 — Gateway/feed orchestration integration
**Read:** IPC/feed/state.
**Allowed:** Rust app/ipc/feed orchestration only.
**Goal:** replace local metadata source with gateway; no renderer change.
**Gate:** fake gateway full app first, then live metadata startup.

## T-064 — Download/probe integration into buffering
**Read:** cache/download/probe/state.
**Allowed:** Rust app orchestration modules only.
**Goal:** selected Reel -> atomic download -> probe -> playable current; recover/skip on one failure.

## T-065 — One-ahead prefetch integration
**Read:** feed/cache/state.
**Allowed:** app/feed/download orchestration.
**Goal:** prefetch one unseen Reel during playback; operation id; preserve unseen prefetch while retracing forward history.
**Gate:** debug counters/log show ready-next before at least some Next actions.

## T-066 — Live real-Reel custom playback gate
**Read:** risk register/acceptance.
**Allowed:** no architecture change; bug fixes only within currently failing module and only after isolating failure.
**Goal:** real Reel custom ANSI text video + audio.
**Manual gate:** recognizable, correct aspect, controls, no URL leak.

## T-067 — Resize/tiny-terminal integration
**Read:** geometry/state/renderer.
**Allowed:** app/display/terminal integration only.
**Goal:** resize updates latest metrics; current generation frozen/clipped/suspended; next Reel re-geometrizes; Quit always works.
**Manual gate:** multiple shrink/grow cycles no scroll/corruption.

## T-068 — Recoverable failure integration
**Read:** error matrix/state.
**Allowed:** app orchestration/error/status.
**Goal:** force one download/probe/decode/audio failure and prove app skips/degrades without state corruption.
**Gate:** each forced case has expected user status and redacted log.

---

# Phase 9 — Hardening and performance

## T-069 — Renderer microbenchmark/counters
**Read:** performance/observability.
**Allowed:** renderer benchmark/dev diagnostic, counter fields.
**Goal:** measure encode duration/bytes at max default geometry in release mode.
**Forbidden:** architecture optimization yet.
**Gate:** recorded local baseline.

## T-070 — Apply only required ANSI performance cleanup
**Prerequisite:** T-069 shows need OR review identifies obvious allocation bug.
**Read:** performance optimization order.
**Allowed:** `renderer.rs`, `ansi.rs`, tests/benchmark.
**Goal:** remove per-cell allocations, reserve/reuse buffer, coalesce SGR, preserve exact output semantics.
**Forbidden:** dirty diff/image protocols/quantization.
**Gate:** tests unchanged + benchmark improved/not regressed materially.

## T-071 — Rapid navigation/pause stress
**Read:** acceptance/state.
**Allowed:** tests/harness; source fixes only in isolated failing module.
**Goal:** scripted/local repeated Next/Previous/Pause with stale generations.
**Gate:** no overlapping display generations, no child leak, no panic.

## T-072 — 30-Reel live soak
**Read:** test strategy/acceptance/risk register.
**Allowed:** QA notes only unless a reproducible bug is isolated.
**Goal:** required live soak sequence.
**Record:** start/end memory estimate, cache count/size, frames dropped summaries, failures, child PIDs, terminal state.
**PASS:** no unbounded growth/leak/corruption.

## T-073 — Security/redaction/cleanup release audit
**Read:** security/acceptance.
**Allowed:** tests/scripts/docs; source fixes only for audit failure.
**Goal:** grep/git/archive path checks, secret sentinel, URL redaction, path deletion tests, terminal injection tests.
**Gate:** all mandatory security acceptance items PASS.

---

# Phase 10 — CI, docs, packaging, ship

## T-074 — CI without Instagram credentials
**Read:** test strategy/release packaging.
**Allowed:** CI config, scripts needed for deterministic local tests.
**Goal:** Python tests + Rust fmt/check/test/clippy + protocol contract + generated media pipeline where environment permits; no live account.
**Gate:** clean CI run green.

## T-075 — Bootstrap/run scripts
**Read:** architecture/config/release.
**Allowed:** `scripts/bootstrap-dev.sh`, `scripts/run.sh`, focused docs.
**Goal:** documented safe venv/dependency setup; run gateway, wait ping, launch Rust without credential env inheritance, cleanup gateway/socket.
**Gate:** clean-clone manual run.

## T-076 — User README
**Read:** product/release/security.
**Allowed:** top-level `README.md`, safe generated-fixture screenshot/GIF assets only if desired.
**Goal:** installation, system deps, credentials/session paths, run, keys, text-renderer explanation, unofficial Instagram warning, troubleshooting links.
**Forbidden:** real account screenshot/private metadata.

## T-077 — Architecture/security/troubleshooting docs
**Read:** current specs and actual shipped behavior.
**Allowed:** `docs/*.md`.
**Goal:** concise user/developer docs matching implementation; include terminal truecolor/audio/ffmpeg/auth challenge troubleshooting.

## T-078 — CHANGELOG/LICENSE/version
**Allowed:** `CHANGELOG.md`, `LICENSE`, version fields/metadata explicitly needed.
**Goal:** v1.0.0 release metadata; no feature code.

## T-079 — Release-check script
**Read:** release packaging/acceptance.
**Allowed:** `scripts/release-check.sh`.
**Goal:** automate clean status, tests, release build, forbidden artifact scan, archive listing validation without reading actual secrets.
**Gate:** script PASS on clean tree.

## T-080 — Clean-clone release rehearsal
**Read:** release packaging.
**Allowed:** QA notes/document corrections only; source bug fix requires separate scoped task.
**Goal:** new clean checkout, documented setup, build, fake tests, generated clip, live manual run, quit cleanup.
**PASS:** another environment/user can follow docs without hidden local state.

## T-081 — Build v1.0.0 release archive
**Read:** release packaging.
**Allowed:** release artifact output only; no code changes.
**Goal:** source archive with required content and no forbidden runtime files.
**Gate:** list archive, run release-check, hash artifact.

## T-082 — Final ship sign-off
**Read:** `spec/13_ACCEPTANCE.md` line by line.
**Allowed:** `RELEASE_QA_v1.0.0.md` or equivalent QA record.
**Goal:** mark every mandatory item PASS with command/manual evidence; NOT RUN means **do not ship**.
**Final condition:** all mandatory checks PASS -> tag/release v1.0.0. Otherwise return to the first failing scoped task; do not waive silently.
