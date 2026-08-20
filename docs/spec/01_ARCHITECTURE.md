# 01 — Architecture

## 1. Runtime components

### Python IG gateway
Owns only:
- instagrapi client/session,
- lazy authentication initialization,
- Reels/Discover request,
- DTO normalization,
- stable upstream error mapping,
- framed MessagePack UDS server.

Does not own media download, render, audio, terminal, cache, or navigation.

### Rust terminal application
Owns:
- config/preflight,
- IPC client,
- metadata queue/dedup,
- HTTPS downloader,
- run cache,
- media probe,
- geometry calculation,
- FFmpeg decoder process,
- RGB frame buffering,
- terminal text renderer,
- audio,
- AV scheduling,
- input,
- App state machine,
- display owner,
- logging,
- cleanup.

## 2. Runtime data flow

```text
Gateway ReelDTO
   |
   v
FeedQueue --select--> ReelMeta
   |                    |
   |                    v
   |                 Downloader
   |                    |
   |                    v
   |                 Local MP4
   |                    |
   |             +------+------+
   |             |             |
   |             v             v
   |          ffprobe        rodio
   |             |             |
   |             v             |
   |          MediaInfo        |
   |             |             |
   |             v             |
   |        RenderGeometry     |
   |             |             |
   |             v             |
   |           ffmpeg          |
   |             | RGB24       |
   |             v             |
   |       FrameScheduler <----+
   |             |
   |             v
   |       HalfBlockRenderer
   |             |
   |          RenderFrame
   |             |
   |             v
   +--------> DisplayOwner -> stdout
```

## 3. Launcher lifecycle

A thin `scripts/run.sh` may:
1. resolve repo/runtime paths,
2. load development `.env` for gateway only,
3. create private directories,
4. start Python gateway,
5. wait for auth-free `ping`,
6. launch Rust without credential environment variables,
7. forward/handle shutdown,
8. stop gateway,
9. remove socket.

The launcher contains no application logic.

## 4. XDG paths

Recommended defaults:

```text
socket  ${XDG_RUNTIME_DIR}/ig-reels-cli/gateway.sock
session ${XDG_STATE_HOME:-$HOME/.local/state}/ig-reels-cli/session.json
cache   ${XDG_CACHE_HOME:-$HOME/.cache}/ig-reels-cli/media
logs    ${XDG_STATE_HOME:-$HOME/.local/state}/ig-reels-cli/logs
```

Runtime/socket dir mode `0700`; session file `0600`.

If `XDG_RUNTIME_DIR` is unavailable, launcher may create a user-private fallback under `/tmp` using uid in the directory name and mode `0700`; the exact fallback must be validated against symlink/path attacks.

## 5. Gateway socket lifecycle

On start:
1. validate parent directory ownership/privacy,
2. if socket absent, bind,
3. if path exists, attempt short ping,
4. if healthy gateway answers, exit `already_running`,
5. otherwise confirm path is a Unix socket inside validated app runtime directory,
6. unlink only that stale socket,
7. bind/listen,
8. process one accepted request synchronously at a time.

Never blindly unlink a configured path.

## 6. Rust concurrency model

Workers send events; App changes state.

```text
input worker -----+
feed worker ------+
download worker --+
probe worker -----+--> bounded AppEvent channel --> App loop
playback worker --+
signal worker ----+
```

Rules:
- no `Arc<Mutex<AppState>>`,
- workers own local mutable state,
- App starts/cancels workers,
- every operation has id/generation,
- stale result -> ignore/log at trace/debug,
- shutdown cancels once and waits for tracked tasks/children.

## 7. Display architecture

After raw/alternate-screen entry, there is one `DisplayOwner`.

Allowed stdout path:

```text
App/playback -> DisplayCommand -> DisplayOwner -> buffered stdout
```

Forbidden:
- direct `println!` from worker,
- logger to stdout,
- FFmpeg inheriting stdout/stderr,
- panic message before terminal restoration if restoration can be attempted.

Frame command includes playback generation. Display owner drops stale generation frames. Control commands and video frames use the two-lane contract in `spec/18_INTERNAL_EVENT_AND_DISPLAY_CONTRACTS.md`: reliable bounded control queue plus latest-value frame slot, with control priority.

## 8. Process ownership

### ffprobe
Short-lived per downloaded file. Stdout captured as bounded JSON; stderr bounded. Timeout required.

### ffmpeg
Long-lived per playback generation. Stdout piped RGB24. Stderr drained concurrently with bounded retained tail. Stdin null. Cancellation kills child and waits.

### mpv
Only smoke task. Owns keyboard. Not linked to final App loop.

## 9. Media state ownership

A playback generation owns:
- current Reel id/path,
- probe result,
- frozen geometry,
- FFmpeg child,
- frame reader/channel,
- rodio player/stream handles,
- timing state,
- cancellation token.

Destroy/cancel old generation before accepting a new generation as active.
