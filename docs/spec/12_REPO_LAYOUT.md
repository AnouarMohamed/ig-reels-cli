# 12 — Target Repository Layout

The existing `docs/` directory is the authoritative engineering specification pack for this project.

It predates implementation and MUST be preserved.

Tasks that create or modify the repository skeleton MUST NOT delete, replace, prune, rename, or reorganize existing files under `docs/` unless a task explicitly requires a documentation change.

## Target repository layout

```text
ig-reels-cli/
├── AGENTS.md
├── README.md
├── LICENSE
├── CHANGELOG.md
├── .gitignore
├── .env.example
│
├── docs/
│   ├── README.md
│   ├── MANIFESTO_V3.md
│   ├── DECISIONS.md
│   ├── CONCEPTION_REVIEW.md
│   ├── HOW_TO_USE_WITH_AI.md
│   ├── RELEASE_QA_TEMPLATE.md
│   ├── RISK_REGISTER.md
│   ├── SHIP_RUNBOOK.md
│   ├── adr/
│   ├── agent/
│   ├── references/
│   ├── scripts/
│   ├── source/
│   ├── spec/
│   └── guides/
│       ├── architecture.md
│       ├── security.md
│       └── troubleshooting.md
│
├── scripts/
├── py-ig-gateway/
│   ├── requirements.txt
│   ├── config.py
│   ├── protocol.py
│   ├── ig_client.py
│   ├── daemon.py
│   └── tests/
│
└── rust-tui/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── src/
    └── tests/
```

## Important distinction

There are two categories of documentation.

### Engineering specification

These files define how the project MUST be built:

```text
docs/MANIFESTO_V3.md
docs/DECISIONS.md
docs/spec/
docs/adr/
docs/agent/
```

They are normative and MUST NOT be replaced by implementation-oriented documentation.

### Human-facing project guides

These explain the finished project to developers and users:

```text
docs/guides/architecture.md
docs/guides/security.md
docs/guides/troubleshooting.md
```

These may be created later by their assigned tasks.

They do not replace the normative engineering specification.

## T-001 preservation rule

For T-001 specifically:

- preserve all existing `docs/` content,
- do not remove any existing documentation file,
- do not rename existing documentation directories,
- create only missing repository skeleton files/directories required by T-001,
- `docs/guides/` may be created if T-001 requires directory scaffolding,
- the three guide files do not need to exist until their assigned documentation tasks unless T-001 explicitly requires placeholders.

If the actual `docs/` directory contains additional specification files not shown above, preserve them.

The tree above describes required structural locations. It is NOT an exhaustive whitelist of allowed files inside `docs/`.

## Ownership boundaries

### `renderer.rs`

Pure RGB frame -> semantic terminal-cell/render-frame conversion.

No stdout.

No FFmpeg.

No app navigation.

### `ansi.rs`

ANSI/terminal byte encoding helpers and color-state encoder.

No remote unsanitized text.

### `display.rs`

Sole stdout writer in TUI mode.

Owns generation filtering, `BeginGeneration` clearing, frame writes, and status writes.

### `geometry.rs`

Pure math only.

No terminal I/O.

Inputs are already measured terminal metrics and media aspect information.

### `media_probe.rs`

ffprobe process boundary and JSON normalization only.

No rendering.

### `decode.rs`

FFmpeg RGB24 child process and whole-frame reader only.

No ANSI.

### `scheduler.rs`

Frame timing and drop policy only.

No navigation ownership.

### `app.rs`

Application state machine and orchestration.

No raw ANSI bytes.

### `terminal.rs`

RAII terminal setup/restore and terminal metric query.

No media logic.

## Anti-patterns forbidden by layout

Do not create:

- `utils.rs` as a dumping ground for unrelated logic,
- a monolithic `player.rs` containing process, audio, render, input, and application state,
- a Python media downloader,
- a renderer that directly writes to stdout,
- an App layer that directly executes FFmpeg command strings.

## Preservation invariant

Existing normative documentation is never considered obsolete merely because it is not explicitly listed in this file.

Repository-layout tasks are additive with respect to `docs/` unless a dedicated documentation migration task explicitly says otherwise.
