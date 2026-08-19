Glossary of Project-Specific Terms

- **IPC boundary**: The communication barrier between the Python auth daemon and the Rust TUI app, implemented via a Unix domain socket where msgpack‑encoded request/response messages are exchanged.
- **Prefetch**: The background downloading of reel N+1 (or further) while reel N is currently playing, to eliminate stalls when advancing to the next reel.
- **UDS (Unix Domain Socket)**: A local inter‑process communication mechanism that allows data exchange between processes on the same host using filesystem paths (e.g., /tmp/ig-reels.sock) instead of network ports.
- **msgpack**: A binary serialization format used to encode structured data (maps, arrays, strings, numbers) for efficient transmission over the UDS; it is language‑agnostic and more compact than JSON.
- **tmp-cache/**: A temporary, git‑ignored directory under the Rust app where downloaded MP4 reel files are stored transiently and removed on normal exit.
- **session.json**: A file created by instagrapi in the Python daemon's directory that stores encrypted Instagram session cookies, allowing reuse of login across daemon restarts.
- **checkpoint/challenge**: An Instagram security mechanism that requires the user to verify their identity (via email, SMS, or CAPTCHA) before granting API access; it appears as a ChallengeRequired exception from instagrapi.
- **ANSI truecolor half‑block rendering**: A technique to display bitmap images in a terminal using the Unicode half‑block character (▀) where the foreground color encodes the top pixel and the background color encodes the bottom pixel, enabling 24‑bit color without graphics protocol support.
- **Kitty graphics protocol**: A terminal‑specific escape‑sequence protocol that allows raster graphics (images) to be displayed inline in compatible terminals (e.g., kitty, iTerm2) with higher fidelity and lower CPU overhead than character‑based rendering.
- **rodio**: A pure‑Rust audio playback crate used to output decoded PCM audio to the system's default sound device.
- **ffmpeg-next**: A Rust wrapper around FFmpeg libraries that enables frame‑accurate video and audio decoding without spawning a subprocess; alternatively, the FFmpeg CLI may be used as a fallback.
- **crossterm**: A cross‑platform Rust library for terminal manipulation, providing raw‑mode input, cursor control, color changes, and querying terminal size.
- **tokio**: An asynchronous runtime for Rust that drives the app's concurrency (IPC, disk I/O, timers) without blocking threads.