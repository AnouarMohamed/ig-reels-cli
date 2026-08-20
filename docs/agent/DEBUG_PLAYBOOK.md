# Debug Playbook — Do Not Random-Walk

Use this only when a task fails.

## IPC failure
1. Confirm fake gateway works with same socket path.
2. Hex/length-check 4-byte frame header.
3. Confirm payload length <=1 MiB.
4. Decode MessagePack independently.
5. Confirm Rust named-map encoding, not tuple struct encoding.
6. Confirm request_id/version.

Do not touch Instagram code for fake-gateway protocol failures.

## Real gateway failure
1. Confirm auth-free ping.
2. Check gateway file log for stable error code.
3. Confirm session path exists/permissions.
4. Inspect current instagrapi docs/source for selected feed call.
5. Do not add scraping/proxy/challenge automation.

## Download failure
1. Log status code/reel id only; do not print URL.
2. Confirm initial and redirect schemes HTTPS.
3. Confirm no cookies required.
4. Check Content-Length/stream cap behavior.
5. If auth cookies are actually required, stop for ADR.

## ffprobe failure
1. Re-run exact argv manually against generated fixture.
2. Capture bounded stderr.
3. Verify JSON fields on installed version.
4. Do not parse pretty stderr as metadata.

## Wrong aspect ratio
1. Print/log safe geometry values: source DAR, terminal cols/rows, cell ratio, sample W/H.
2. Recalculate normative formula manually.
3. Check sample height even.
4. Check orientation agreement.
5. Do not “fix” by stretching FFmpeg output.

## Renderer wrong colors
1. Test literal 1×2 frame.
2. Verify RGB byte order is RGB24, not BGR.
3. Verify top->FG, bottom->BG.
4. Verify SGR 38;2 and 48;2 ordering.
5. Verify status reset does not bleed.

## Terminal scroll/flicker
1. Confirm line wrap disabled.
2. Confirm no newline row movement.
3. Confirm explicit cursor positioning.
4. Confirm frame is one/few buffered writes.
5. Confirm no other stdout writer/logging.
6. Do not add image protocol.

## Playback drift
1. Compare decoded/presented/dropped frame counters.
2. Verify target index uses master clock.
3. Verify stale frames are dropped.
4. Verify pause freezes audio/video clock consistently.
5. If rodio position itself unsuitable, stop for ADR.

## Quit leaves terminal broken
1. Check TerminalGuard creation/destruction path.
2. Check panic hook ordering.
3. Check DisplayOwner/stdout task shutdown.
4. Check child process inheritance.
5. This is a release blocker; do not ignore.
