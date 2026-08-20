# 03 — Feed, Navigation, Download, and Cache

## 1. Domain objects

`ReelMeta` contains at minimum:
- `id`,
- secret/redacted URL wrapper,
- sanitized-safe metadata source fields (raw caption remains data, sanitized at display),
- username,
- optional like count.

The URL wrapper must implement Debug without the URL content.

`CachedReel` adds final local path and byte size.

## 2. In-session id sets

Maintain `seen_ids` for the whole process session. Once accepted into the app from any batch, a Reel id is considered seen and must not be re-added from future batches.

This means Previous/Next history does not require removing ids from `seen_ids`.

## 3. Unseen metadata queue

Target 12, low-water 4.

At most one feed fetch operation may be active.

When queue length < 4 and no feed fetch active, request enough data to restore target; gateway batch call remains capped at 24.

Filter duplicates before insertion.

## 4. Navigation model

State:

```text
back: VecDeque<CachedReel>
current: Option<CachedReel>
forward: VecDeque<CachedReel>
unseen_ready: Option<CachedReel>   # downloaded prefetch
unseen_metadata: VecDeque<ReelMeta>
```

**Deque orientation is fixed:** for both `back` and `forward`, the item nearest to `current` is at the **back** of the deque. Therefore navigation uses `push_back`/`pop_back`. The front is the farthest item on that side.

Back+forward total non-current cached visited Reel count <= 5.

### Next

If `forward` nonempty:
1. `back.push_back(current)`,
2. `current = forward.pop_back()`,
3. do not consume `unseen_ready`.

Else:
1. `back.push_back(current)`,
2. promote `unseen_ready` if available,
3. otherwise buffer/download next metadata,
4. `forward.clear()` because a new branch is consumed.

### Previous

If `back` nonempty:
1. `forward.push_back(current)`,
2. `current = back.pop_back()`,
3. keep `unseen_ready` separate.

If no back item: no-op + optional status hint.

Define stack orientation in unit tests; do not rely on comments alone.

## 5. History eviction

After every navigation mutation:

```text
while back.len + forward.len > 5:
    if back.len >= forward.len and back is not empty:
        evict back.pop_front()      # farthest previous item
    else:
        evict forward.pop_front()   # farthest forward item
```

This preserves the nearest items on both sides and is deterministic.

Eviction deletes only that cached final media file if it belongs to current run and is not referenced by current/unseen_ready.

A simpler implementation may retain file until run shutdown if total cache remains bounded by the same maximum count; state tests still enforce logical bound.

## 6. Download contract

Input must be HTTPS URL.

Reqwest client:
- connect timeout 10 s,
- total request timeout 60 s,
- custom redirect policy max 5,
- each next URL scheme must be HTTPS,
- no cookie jar,
- no Instagram auth/session headers,
- no URL logging.

If `Content-Length` > 150 MiB, reject before body download.

Stream body and count bytes; abort if streamed bytes exceed 150 MiB even when header absent/false.

## 7. Atomic path

Filename derives from a safe encoding/hash of Reel id. Never use raw remote path text.

```text
<run-dir>/<safe-id>.<nonce>.part
<run-dir>/<safe-id>.mp4
```

On success:
1. close writer,
2. flush,
3. rename same-filesystem `.part` -> final,
4. only then publish `DownloadReady`.

On cancellation/error: best-effort remove `.part`.

## 8. Per-run cache ownership

Cache root contains app-owned directories only:

```text
media/
  run-<pid>-<random>/
```

Run directory mode `0700` where supported.

Normal client shutdown deletes only its own run directory.

Startup stale cleanup may remove `run-*` directories older than 24 h only after:
- verifying direct child of configured cache root,
- verifying name pattern,
- refusing symlink traversal,
- never deleting cache root.

## 9. Prefetch

Exactly one unseen downloaded Reel ahead.

When current starts and `forward` is empty, ensure `unseen_ready` is being prepared if metadata exists.

When forward history is nonempty, unseen prefetch may remain ready but must not be overwritten by retracing forward history.

Only one active prefetch download operation. Operation id makes stale completion harmless.

## 10. Failure policy

For one Reel download/probe/playback failure:
- record recoverable error,
- mark failed id seen,
- remove incomplete file,
- attempt next unseen Reel,
- do not retry same URL indefinitely.

If feed cannot replenish and no playable current/forward remains, transition to recoverable/fatal no-content state according to error type.
