# 02 — IPC Protocol v1

## 1. Transport

Unix domain `SOCK_STREAM`.

One client connection carries exactly one request and one response, then both sides close the connection.

## 2. Framing

A logical message is:

```text
4-byte unsigned big-endian payload_length
payload_length bytes of MessagePack
```

Payload limit: `1_048_576` bytes.

Reject before allocation if declared length is zero or over limit.

Read exactly 4 header bytes, then exactly payload length. Short EOF is `protocol_truncated`.

## 3. Serialization rules

- maps use string keys,
- UTF-8 strings decode as strings, not raw byte strings,
- Rust structs serialize as named-field maps (`rmp-serde` named encoding),
- unknown response fields may be ignored for forward compatibility,
- required fields must exist,
- request/response `protocol_version` is integer `1`.

Python MessagePack configuration baseline:
- `packb(..., use_bin_type=True)`
- `unpackb(..., raw=False, strict_map_key=True)`

## 4. Request envelope

```json
{
  "protocol_version": 1,
  "request_id": "opaque-local-id",
  "cmd": "ping|get_reels",
  "args": {}
}
```

`request_id` maximum 64 ASCII printable characters; never contains secrets.

### ping

```json
{
  "protocol_version": 1,
  "request_id": "r1",
  "cmd": "ping",
  "args": {}
}
```

### get_reels

```json
{
  "protocol_version": 1,
  "request_id": "r2",
  "cmd": "get_reels",
  "args": {"count": 12}
}
```

Accepted count range: 1–24. Values outside return `invalid_request`.

## 5. Success envelope

```json
{
  "protocol_version": 1,
  "request_id": "r1",
  "ok": true,
  "result": {...}
}
```

Ping result:

```json
{"service":"ig-gateway","status":"ok"}
```

Get reels result:

```json
{
  "items": [
    {
      "id":"...",
      "video_url":"https://...",
      "caption":"... or null",
      "username":"...",
      "like_count":1234
    }
  ]
}
```

`like_count` may be null if unavailable.

## 6. Error envelope

```json
{
  "protocol_version": 1,
  "request_id": "r2",
  "ok": false,
  "error": {
    "code": "challenge_required",
    "detail": "sanitized human-readable text"
  }
}
```

Stable v1 codes:
- `invalid_request`
- `unsupported_protocol`
- `unknown_command`
- `auth_required`
- `challenge_required`
- `login_failed`
- `upstream_rate_limited`
- `upstream_unavailable`
- `upstream_changed`
- `internal_error`
- `already_running` (startup-level, when applicable)

Error detail must not include credentials, session content, cookie values, or full signed URL.

## 7. ReelDTO constraints

| field | rule |
|---|---|
| `id` | nonempty string, max 128 chars |
| `video_url` | nonempty HTTPS URL string, max 8192 chars |
| `caption` | null or string, max 20,000 chars before Rust display truncation |
| `username` | nonempty string, max 256 chars |
| `like_count` | null or nonnegative integer |

Gateway normalizes upstream weirdness; Rust validates again at trust boundary.

## 8. Timeouts

Rust:
- connect timeout 2 s,
- whole request/response 30 s.

Gateway must not hold a connection indefinitely. Upstream failures map to response when possible.

## 9. Contract tests

Mandatory:
- Python encode -> Rust decode,
- Rust named-map encode -> Python decode,
- header split across reads,
- payload split across reads,
- zero size,
- oversize declared size,
- truncated payload,
- invalid MessagePack,
- unsupported version,
- wrong type for count,
- response request_id mismatch rejected by client.
