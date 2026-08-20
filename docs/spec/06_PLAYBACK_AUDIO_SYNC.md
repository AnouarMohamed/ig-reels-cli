# 06 — Playback, Decode, Audio, and AV Sync

## 1. Playback generation

Every started Reel receives a monotonically increasing `playback_generation: u64`.

Generation owns:
- cached file path,
- `MediaInfo`,
- `RenderGeometry`,
- FFmpeg child,
- RGB reader/channel,
- stderr drainer,
- rodio output/player handles if available,
- clock state,
- cancellation token.

No event/frame from generation G may affect App or DisplayOwner after active generation != G.

## 2. FFmpeg decode process

Input: final local MP4.

Output: raw `rgb24` frames to stdout.

Conceptual argument shape:

```text
ffmpeg
-v error
-i <file>
-an
-vf fps=<FPS>,scale=<W>:<H>
-pix_fmt rgb24
-f rawvideo
pipe:1
```

Important: `W` and `H` are already the exact content sample dimensions calculated by Rust to preserve physical display aspect. Do not ask FFmpeg to stretch to a full terminal canvas.

The dedicated task must verify actual filter syntax and orientation behavior against the pinned/installed FFmpeg.

Process settings:
- stdin null,
- stdout piped,
- stderr piped and drained,
- no shell,
- timeout applies to startup/probe, not arbitrary short whole-playback kill,
- cancellation kills child and waits.

## 3. RGB frame reading

```text
frame_bytes = W * H * 3
```

Use checked arithmetic and a sane upper bound derived from configured max columns/rows.

Read exactly `frame_bytes` for one frame.

Cases:
- exactly 0 bytes at frame boundary + child clean exit -> EOF,
- partial frame then EOF -> decoder/truncated error; do not render partial frame,
- child nonzero -> playback failure with bounded stderr tail,
- cancellation -> cancelled result, not user-visible error.

## 4. Bounded frame channel

Capacity = 3 frames.

Decoder can backpressure when renderer/scheduler falls behind. It never stores a Reel of raw frames.

Each frame:

```text
DecodedFrame {
  generation,
  frame_index,
  rgb
}
```

## 5. Nominal video timestamps

At 15 FPS:

```text
frame_interval = 1 / 15 s
pts(frame_index) = frame_index / 15 s
```

Frames are numbered from 0.

## 6. Audio spike before integration

Before building app-level sync, prove current pinned rodio path on:
1. generated MP4/AAC fixture,
2. real downloaded Reel.

Required behavior:
- open default sink/device,
- decode file through current supported rodio Decoder path,
- create Player connected to mixer/device,
- append source,
- pause/play works,
- `get_pos()` advances during playback and stops/fixes appropriately while paused.

If this fails on real Reel media, stop for ADR.

## 7. Audio-capability state

Session-level:

```text
Unknown
Available
Unavailable(reason)
```

If opening default audio device fails once, mark session `Unavailable`; do not reopen every frame/Reel automatically.

Per Reel:
- no audio stream -> video-only clock,
- audio stream + session available + decode succeeds -> audio-master,
- audio decode fails -> recoverable video-only for that Reel plus visible status/log.

## 8. Start sequence with audio

Target sequence:
1. probe file,
2. compute geometry,
3. prepare rodio player paused and append source,
4. spawn FFmpeg,
5. read/obtain first decoded frame or confirm decoder running,
6. issue DisplayOwner `BeginGeneration`,
7. establish `start_instant`,
8. start rodio player,
9. scheduling loop presents frames according to audio position.

Do not start audio several seconds before first video frame is available.

## 9. Audio-master scheduler

At each scheduling opportunity:

```text
master_pos = player.get_pos()
target_index = floor(master_pos_seconds * FPS)
```

Maintain newest decoded frame at/before or near target.

Rules:
- if decoded frame index < target and newer frames are already available, drop stale frames,
- present newest appropriate frame,
- do not pause audio to wait for a late frame,
- do not busy-spin; use bounded sleep/select/event wait,
- frame display is idempotent by index; do not intentionally present same frame repeatedly unless no new frame and display state requires it.

A small tolerance may be introduced only by a documented constant and tests. Initial implementation should be simple and measured.

## 10. Video-only scheduler

Use monotonic `Instant`.

State tracks:
- `started_at`,
- accumulated paused duration,
- pause start if paused.

Effective position excludes paused time.

Same target-index formula.

## 11. Pause

On Space while Playing:
- App enters Paused,
- player.pause() if audio-master,
- video-only clock records pause point,
- scheduler stops presenting advancing frames,
- decoder may block on bounded channel.

Resume:
- player.play() or resume monotonic clock,
- scheduler drops now-stale queued video frames as needed.

## 12. Navigation and cancellation

Next/Previous/Quit:
1. App marks old generation no longer active,
2. DisplayOwner receives generation change/clear command before old frame can display,
3. cancel old playback token,
4. stop audio player/source ownership,
5. kill/wait FFmpeg if still alive,
6. ignore stale completion events,
7. then begin new generation.

Order may be implemented through controller primitives but must preserve stale-frame safety.

## 13. Natural completion

Playback is considered naturally ended when video decoder reaches clean EOF and audio is absent or has ended. Avoid hanging forever if one side reports malformed duration; use a bounded tail policy decided in implementation task and tested with fixtures.

Natural completion emits `PlaybackEnded(generation)`.

## 14. Sync acceptance

Generated fixture with visible timing marker + audio should run for >=20 seconds.

Mandatory observations/tests:
- no continuously increasing lag,
- pause for 3 seconds then resume without permanent 3-second AV offset,
- decoder falling behind causes video frame drops rather than audio stall,
- no child/process leak after immediate Next/Quit.
