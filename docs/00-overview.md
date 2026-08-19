IG-REELS-CLI is a terminal-based application for browsing and watching Instagram Reels directly in the terminal. It logs into Instagram (via an unofficial Python daemon), fetches reels, downloads videos, and renders them inline with audio using hand-rolled terminal graphics and audio playback.

Scope:
- Single-user, local, terminal-only application.
- Reads Instagram Explore/hashtag reels feed (read-only).
- Plays video+audio in the terminal using ANSI truecolor or Kitty graphics protocol.
- Supports navigation (next/previous), pause/resume, and quit.
- Shows a thin status line with username, caption, like count.
- Persists login session between runs.

Explicitly out of scope:
- Any web UI or graphical interface (beyond the terminal).
- Multi-user support or synchronization.
- Posting, liking, commenting, following, or any interaction beyond consumption.
- Stories, DMs, IGTV, or any non-Reels content.
- Database persistence beyond temporary video cache.
- Background daemonization or service mode.
- Automated solving of Instagram login challenges/checkpoints.

Target user: a single developer (the user) who wants to watch Instagram Reels without leaving the terminal.

Constraints:
- Entirely terminal-based; no external GUI windows.
- No web backend; all processing happens locally.
- Instagram access is via the unofficial instagrapi library (subject to breakage).
- Must handle login challenges manually (human-in-the-loop).
- Video rendering must be done via CPU-based frame extraction (ffmpeg) and terminal blitting.
- Audio playback must be synchronized with video frames using rodio.
- All secrets (credentials) must be sourced from environment variables, never hardcoded.