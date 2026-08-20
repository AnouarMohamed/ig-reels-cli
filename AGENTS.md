# IG-Reels-CLI — Agent Operating Rules

These rules are mandatory for every coding task in this repository.

The `docs/` directory is the authoritative engineering specification for this project.

Do not treat the current implementation as more authoritative than `docs/`.

---

# 1. SOURCE OF TRUTH

Before changing code, always inspect:

1. `docs/README.md`
2. `docs/MANIFESTO_V3.md`
3. `docs/DECISIONS.md`
4. `docs/agent/TASK_CARDS.md`
5. every specification explicitly referenced by the current task card

Do not begin implementation before doing this.

At the beginning of your response, state:

- current task ID
- task title
- specification files read
- files you expect to modify

If you have not read the relevant specification, STOP.

---

# 2. DOCUMENT PRECEDENCE

When instructions conflict, use this precedence:

1. explicit instruction from the human in the current task
2. `docs/DECISIONS.md`
3. relevant normative file under `docs/spec/`
4. `docs/MANIFESTO_V3.md`
5. current task card
6. existing code
7. your own assumptions

If two normative documents contradict each other:

STOP.

Report the contradiction.

Do not choose one yourself.

---

# 3. DO NOT SILENTLY CHANGE THE ARCHITECTURE

Architecture is frozen unless the human explicitly approves a change.

In particular, do not replace:

- Rust as the main application
- Python/instagrapi as the Instagram adapter
- UDS + length-prefixed MessagePack IPC
- FFmpeg/ffprobe media boundary
- ANSI truecolor rendering
- Unicode half-block `▀` rendering
- Crossterm terminal ownership
- Tokio orchestration

Do not introduce:

- Kitty graphics
- Sixel
- iTerm image protocol
- GUI windows
- SDL
- OpenGL
- web rendering
- embedded browser video
- PyO3

unless an explicit architecture task authorizes it.

The final video must remain terminal text rendered through Unicode characters and ANSI escape sequences.

---

# 4. DOCS ARE NOT A WAY TO MAKE CODE PASS

Do not modify `docs/` simply because the implementation is easier another way.

Do not weaken a requirement.

Do not change an acceptance criterion to match broken code.

Do not remove a test because it fails.

Only edit `docs/` when:

- the current task explicitly requires documentation changes, or
- the human explicitly approves an architecture/specification change.

If implementation and documentation disagree, assume the implementation is wrong until proven otherwise.

---

# 5. ONE TASK AT A TIME

Only implement the task ID explicitly assigned by the human.

Example:

T-017

means:

implement T-017 only.

Do not start T-018.

Do not perform unrelated refactoring.

Do not clean neighboring modules.

Do not add speculative abstractions for future tasks.

Do not implement "while I am here" improvements.

A task is complete only when its acceptance criteria pass.

---

# 6. REQUIRED TASK START PROCEDURE

Before editing anything:

Run:

```bash
git status --short
git branch --show-current
git log -5 --oneline
```

Verify:

- working tree is understood
- current branch is not accidentally carrying unrelated changes
- previous task state is known

Then read the current task card and referenced specs.

Then write a short implementation plan.

Only then modify code.

---

# 7. BRANCH POLICY

Never implement directly on `main`.

Each task uses its own branch:

```text
task/T-001-repository-skeleton
task/T-002-python-framing
task/T-003-fake-gateway
...
```

Branch naming format:

```text
task/T-NNN-short-description
```

One task per branch.

Never mix multiple task IDs in the same branch.

---

# 8. COMMIT POLICY

Goal:

ONE COMPLETED TASK = ONE CLEAN COMMIT

Final commit format:

```text
<type>(T-NNN): <short description>
```

Examples:

```text
chore(T-001): scaffold project structure
feat(T-008): implement framed msgpack transport
test(T-014): add geometry golden cases
fix(T-031): restore terminal after playback failure
docs(T-078): document release procedure
```

Allowed types:

```text
feat
fix
test
refactor
docs
chore
build
ci
perf
```

Do not create commits such as:

```text
stuff
changes
fix
fix again
oops
wip
update
more fixes
final
final2
```

Do not commit knowingly broken code.

Do not commit debugging output.

Do not commit commented-out experiments.

Do not commit generated temporary files.

---

# 9. DURING A TASK

You may make temporary local edits while solving the task.

Before the final commit:

- remove debugging code
- remove temporary files
- remove dead experiments
- run formatting
- run linting
- run the task's required tests
- inspect the final diff

Use:

```bash
git diff
git diff --stat
git status --short
```

Check every changed file.

If a changed file is unrelated to the current task, revert that change.

---

# 10. TEST POLICY

Never claim a test passed unless you actually executed it.

Report exact commands.

Distinguish:

PASS
FAIL
NOT RUN

`NOT RUN` is not equivalent to PASS.

If required tests cannot run, explain why and do not declare the task complete.

Never alter tests merely to hide implementation failures.

---

# 11. RUST QUALITY GATE

When Rust code is affected, run the appropriate repository commands including, when available:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

If the workspace layout requires a different command, use the documented repository command.

Do not invent exceptions to failing Clippy warnings.

---

# 12. PYTHON QUALITY GATE

When Python code is affected, run the project's documented Python checks.

Expected checks include, when configured:

```bash
ruff check .
ruff format --check .
pytest
```

Do not introduce additional Python tooling without authorization.

---

# 13. SECURITY RULES

Never print or commit:

- Instagram username
- Instagram password
- session cookies
- session.json
- authorization headers
- complete signed media URLs
- secrets from `.env`

Before every commit inspect:

```bash
git diff --cached
```

for accidental secrets.

Never add `.env` or session files to Git.

---

# 14. PROCESS BOUNDARIES

Every external process interaction must explicitly handle:

- spawn failure
- non-zero exit
- timeout where applicable
- malformed output
- unexpected EOF
- cancellation
- cleanup

Relevant processes include:

- Python gateway
- ffmpeg
- ffprobe
- mpv smoke-test process

Do not silently ignore subprocess failures.

---

# 15. TERMINAL OWNERSHIP

Only the designated display owner may write playback ANSI output to terminal stdout.

Background tasks must not directly render frames.

Do not create multiple competing stdout writers.

Terminal state must be restored on:

- normal quit
- playback failure
- task cancellation
- recoverable application error
- panic paths where restoration is possible

---

# 16. PERFORMANCE RULE

Do not "optimize" before measurements exist.

For v1 renderer correctness, follow the documented pipeline exactly.

Do not introduce:

- SIMD
- GPU acceleration
- alternate Unicode encoders
- quadrant rendering
- Braille rendering
- terminal graphics protocols
- frame-diff rendering

unless the relevant task explicitly requests it.

---

# 17. WHEN YOU MUST STOP

STOP and ask the human when:

- normative specs contradict each other
- the task requires an architecture change
- Instagram auth/challenge behavior requires modifying protected auth logic
- a required library/API appears unavailable
- implementation requires weakening an acceptance criterion
- an unrelated existing failure prevents verification
- a destructive Git operation seems necessary
- secrets may have entered Git history

Do not improvise around these conditions.

---

# 18. GIT SAFETY

Never run destructive Git commands without explicit human authorization.

Forbidden unless explicitly requested:

```bash
git reset --hard
git clean -fd
git push --force
git push --force-with-lease
git rebase --onto
git filter-repo
git filter-branch
```

Never rewrite `main`.

Never delete another task's work.

---

# 19. TASK COMPLETION PROCEDURE

Before declaring a task finished:

1. reread the task acceptance criteria
2. run required tests
3. run formatting/linting
4. inspect `git diff`
5. inspect `git status`
6. verify no unrelated files changed
7. verify no secrets exist
8. create the task commit
9. show the resulting commit hash

Then report:

```text
TASK: T-NNN
STATUS: COMPLETE

FILES CHANGED:
...

TESTS:
PASS  ...
PASS  ...

COMMIT:
<hash> <message>

DOC DEVIATIONS:
none

KNOWN ISSUES:
none
```

Do not begin the next task.

Wait for the human to assign it.

---

# 20. FINAL RULE

When uncertain:

READ THE DOCS AGAIN.

Do not guess.

Do not redesign.

Do not skip ahead.

Do not silently relax requirements.

The objective is not to produce code quickly.

The objective is to produce the implementation specified in `docs/`, one verified task at a time.
