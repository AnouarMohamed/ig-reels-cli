# 17 — Terminal Compatibility Policy

## 1. Required terminal behavior

v1.0 expects:
- UTF-8 text,
- cursor addressing/alternate screen/raw mode compatible with Crossterm,
- ANSI/xterm-style 24-bit SGR color handling,
- at least 20×8 character cells.

## 2. Truecolor detection is not treated as perfectly discoverable

There is no single portable, authoritative capability query for every terminal/multiplexer combination. Therefore the application must not maintain a giant guessed terminal-name allowlist.

Classify:

```text
Unsupported
LikelyTruecolor
Unknown
```

### Unsupported
- `TERM=dumb`, or
- Crossterm cannot establish required terminal operations.

Exit before playback with a clear message.

### LikelyTruecolor
If `COLORTERM`, case-insensitive, is exactly/contains conventional `truecolor` or `24bit` signal, classify likely.

### Unknown
Anything else that is otherwise a functioning ANSI terminal.

For `Unknown`, continue using truecolor SGR because truecolor is the product requirement, but show/log a one-time warning before TUI entry:

```text
truecolor capability not advertised; continuing with 24-bit ANSI output
```

Do not silently downgrade to 256 colors.

## 3. Manual compatibility test

Provide a development command/script that renders:
- red/green/blue blocks,
- a smooth RGB gradient,
- several `▀` cells with different FG/BG colors.

This test contains no image protocol.

## 4. Multiplexers

Do not add tmux/screen-specific passthrough or image logic in v1.0. If standard ANSI truecolor is altered by a user's multiplexer configuration, document it as environment configuration rather than bypassing with graphics protocols.

## 5. Supported acceptance environments

Before ship, manually test at least two terminal emulators available to the project owner. Record exact terminal names/versions in release QA, not as hard-coded runtime allowlist.
