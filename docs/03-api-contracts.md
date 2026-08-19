Python Auth Daemon API Contracts
Communication over Unix Domain Socket (UDS) using MessagePack (msgpack) encode/decode.
All messages are maps (dictionaries). Keys are strings; values are primitives or nested maps.

----------------------------------------------------------------
Request Messages
----------------------------------------------------------------

1. Ping
   - Purpose: Verify daemon is responsive.
   - Map: {"cmd": "ping"}
   - No additional fields.

2. Get Reels Batch
   - Purpose: Request a batch of reel metadata.
   - Map: {"cmd": "get_reels", "count": <positive integer>}
   - "count": Number of reels to return (e.g., 1‑10). If omitted, defaults to 1.

----------------------------------------------------------------
Response Messages
----------------------------------------------------------------

All responses are maps with a top-level key indicating outcome.

A. Success Responses
   1. Ping Success
        Map: {"status": "ok"}
   2. Get Reels Success
        Map: {
          "reels": [
            {
              "id": "<string>",           // Instagram media ID
              "video_url": "<string>",    // Direct MP4 URL (https://...)
              "caption": "<string>",      // Caption text, empty if none
              "username": "<string>",     // Owner's username
              "like_count": <integer>     // Number of likes (may be 0)
            },
            ... up to 'count' items ...
          ]
        }

B. Error Responses
   Daemon returns error maps with "error" key and optional "detail".
   1. Unknown Command
        Map: {
          "error": "unknown_command",
          "detail": "<string>"   // e.g., "Unknown command: foo"
        }
   2. Invalid Request (Malformed msgpack or missing fields)
        Map: {
          "error": "invalid_request",
          "detail": "<string>"   // e.g., "failed to unpack msgpack"
        }
   3. Internal Error (unexpected exception in daemon)
        Map: {
          "error": "internal_error",
          "detail": "<string>"   // e.g., traceback or error message
        }
   4. Challenge Required (instagrapi checkpoint/challenge)
        Map: {
          "error": "challenge_required",
          "detail": "<string>"   // Exception message from instagrapi
        }
        // The user must solve the challenge via Instagram website/app,
        // then restart the daemon with a fresh session (delete session.json).

Note: Msgpack encoding uses binary format; the above JSON representations
illustrate the structure. Byte‑order and msgpack specifics (use_bin_type=true)
are handled by the msgpack library; applications should not rely on
field ordering.