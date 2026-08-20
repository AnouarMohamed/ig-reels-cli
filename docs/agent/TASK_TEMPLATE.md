# Task Card Template for Any Necessary Follow-up Task

Use only when a bug/ADR creates work not already represented by T-001..T-082.

```text
## T-NEW — <single narrow outcome>

**Reason:** <bug/ADR/evidence that requires this task>
**Prerequisites:** <passing task ids>
**Read:** <exact specs/ADRs>
**Allowed:** <exact file paths/globs>
**Goal:** <one observable outcome>
**Required:**
- <specific behavior>
- <specific behavior>
**Tests:**
- <exact automated proof>
**Manual gate:** <if truly required>
**Forbidden:**
- <adjacent scope>
**Stop condition:** <what evidence requires ADR instead>
```

A follow-up task must not be used to hide unrelated refactoring.
