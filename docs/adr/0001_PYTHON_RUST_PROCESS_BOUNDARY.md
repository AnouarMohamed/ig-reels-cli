# ADR-0001 — Keep Instagram Access in a Separate Python Process

**Status:** accepted

## Context
`instagrapi` solves the unstable unofficial Instagram access problem. The main application benefits from Rust for process/media/terminal/concurrency work.

## Decision
Keep Python gateway and Rust terminal app as separate OS processes connected by framed MessagePack over a Unix domain socket.

## Consequences
- Python crash/auth issue does not corrupt terminal process directly.
- Rust does not embed Python/runtime/credentials.
- IPC contract must be versioned/tested.
- Linux/Unix process model is the v1.0 target.
