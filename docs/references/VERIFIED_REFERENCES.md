# Verified Technical References

Checked during v3 conception on **2026-08-20**. These references justify specific implementation assumptions. They do not replace compiler verification against the pinned dependency.

## instagrapi

Repository / README:
https://github.com/subzeroid/instagrapi

Verified conception facts:
- unofficial/private Instagram client,
- Python session persistence examples use `dump_settings`, `load_settings`, and `login`,
- saved-session reuse is an intended flow,
- Reels-related support exists,
- upstream/private API behavior can change and must be isolated.

The project pins `instagrapi==2.18.16` as its conception baseline from the preceding verified pack. Re-check package availability when creating the actual lock environment.

## mpv text-console renderer

Stable manual:
https://mpv.io/manual/stable/

Verified facts:
- `tct` is a color Unicode-art output for text consoles,
- truecolor is its normal full-color path,
- half-block algorithm exists,
- output buffering can be pixel/line/frame,
- terminal output interleaving can break the image,
- this supports our architectural choice of a single output owner and frame buffering.

mpv also documents Kitty separately, reinforcing that Kitty is an image/graphics protocol path and is intentionally forbidden for the final product.

## Crossterm

Terminal size:
https://docs.rs/crossterm/latest/crossterm/terminal/fn.size.html

Terminal module / `window_size` source notes:
https://docs.rs/crossterm/latest/src/crossterm/terminal.rs.html

Event module:
https://docs.rs/crossterm/latest/src/crossterm/event.rs.html

Verified facts:
- `size()` returns columns/rows,
- `window_size()` exists,
- pixel width/height may be unreliable or zero on Unix,
- event `poll()` allows readiness checking before blocking `read()`.

These facts motivate cell-aspect fallback and cancellable poll/read input.

## xterm control sequences / truecolor semantics

Control sequences:
https://invisible-island.net/xterm/ctlseqs/ctlseqs.html

FAQ discussing SGR 38/48 color extensions:
https://invisible-island.net/xterm/xterm.faq.html

The implementation uses conventional truecolor SGR semantic forms `38;2;R;G;B` and `48;2;R;G;B`. Exact emitted bytes are unit-tested.

## FFmpeg

Main docs:
https://ffmpeg.org/ffmpeg.html

Filters:
https://ffmpeg.org/ffmpeg-filters.html

Formats/rawvideo:
https://ffmpeg.org/ffmpeg-formats.html

Verified facts:
- FFmpeg CLI can filter/scale frame streams,
- rawvideo has no header and therefore dimensions/pixel format are an external contract,
- `rgb24` raw frame output over a pipe is a viable process boundary.

## ffprobe

https://ffmpeg.org/ffprobe.html

Verified facts:
- ffprobe emits machine-readable JSON,
- `-show_entries` can restrict fields,
- stream metadata can be inspected without parsing human diagnostic output.

## rodio

Main docs:
https://docs.rs/rodio/latest/rodio/

Player:
https://docs.rs/rodio/latest/rodio/struct.Player.html

Decoder:
https://docs.rs/rodio/latest/rodio/decoder/struct.Decoder.html

Verified current conception facts:
- rodio uses a `Player` abstraction,
- `Player::get_pos()` reports playback position,
- pause/play controls exist,
- common default decoders include MP4 via Symphonia configuration in the current crate docs,
- `Decoder::try_from(File)` is documented as a preferred optimized file path.

The real-Reel audio spike remains mandatory because container/codec reality must be proven, not assumed.

## Tokio cancellation

https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html

Verified fact: `CancellationToken` provides explicit cancellation signaling suitable for worker/generation shutdown.

## Reqwest

https://docs.rs/reqwest/latest/reqwest/
https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
https://github.com/seanmonstar/reqwest/blob/master/src/redirect.rs

Verified facts:
- client-level connect/whole-request timeouts exist,
- redirects are customizable,
- custom redirect policies must implement their own loop/hop policy rather than assuming default behavior,
- TLS is the normal HTTPS path.

## rmp-serde

https://docs.rs/rmp-serde/latest/rmp_serde/
https://docs.rs/rmp-serde/latest/src/rmp_serde/lib.rs.html

Verified fact: `to_vec_named` is exported and is required by this cross-language contract to encode Rust struct field names as maps instead of compact tuple-like struct representation.
