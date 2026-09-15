#!/usr/bin/env python3
"""
Fake Instagram Gateway Server for IG-Reels-CLI.

Implements a mock Unix Domain Socket (UDS) server that adheres to the framed
MessagePack IPC protocol (v1). Serves auth-free `ping` and deterministic `get_reels`
responses, with support for selectable error simulation.
"""

import argparse
import os
import pathlib
import signal
import socket
import sys
from typing import Any, Dict, List, Optional

# Ensure py-ig-gateway is in Python path to import protocol helpers
REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
PY_GATEWAY_DIR = REPO_ROOT / "py-ig-gateway"
if str(PY_GATEWAY_DIR) not in sys.path:
    sys.path.insert(0, str(PY_GATEWAY_DIR))

from protocol import decode_frame, encode_frame  # noqa: E402

DEFAULT_SOCKET_PATH = "/tmp/ig-reels-fake-gateway.sock"

FAKE_REELS: List[Dict[str, Any]] = [
    {
        "id": "fake_reel_001",
        "video_url": "https://example.com/reels/fake_video_001.mp4",
        "caption": "First fake reel for testing terminal player UI and IPC.",
        "username": "fake_creator_one",
        "like_count": 1234,
    },
    {
        "id": "fake_reel_002",
        "video_url": "https://example.com/reels/fake_video_002.mp4",
        "caption": "Second fake reel with high quality example content.",
        "username": "fake_creator_two",
        "like_count": 5678,
    },
    {
        "id": "fake_reel_003",
        "video_url": "https://example.com/reels/fake_video_003.mp4",
        "caption": "Third fake reel caption without likes.",
        "username": "fake_creator_three",
        "like_count": None,
    },
]


def handle_request(
    req: Dict[str, Any], simulated_error: Optional[str] = None
) -> Dict[str, Any]:
    """Process an incoming IPC request dict and return a response dict."""
    protocol_version = req.get("protocol_version")
    request_id = req.get("request_id", "")
    cmd = req.get("cmd")
    args = req.get("args", {})

    if protocol_version != 1:
        return {
            "protocol_version": 1,
            "request_id": request_id,
            "ok": False,
            "error": {
                "code": "unsupported_protocol",
                "detail": f"Unsupported protocol_version: {protocol_version}",
            },
        }

    if not isinstance(request_id, str) or not request_id:
        return {
            "protocol_version": 1,
            "request_id": str(request_id),
            "ok": False,
            "error": {
                "code": "invalid_request",
                "detail": "request_id must be a non-empty string",
            },
        }

    if cmd == "ping":
        return {
            "protocol_version": 1,
            "request_id": request_id,
            "ok": True,
            "result": {
                "service": "ig-gateway",
                "status": "ok",
            },
        }

    if cmd == "get_reels":
        if simulated_error:
            return {
                "protocol_version": 1,
                "request_id": request_id,
                "ok": False,
                "error": {
                    "code": simulated_error,
                    "detail": f"Simulated error mode active: {simulated_error}",
                },
            }

        count = args.get("count", 12) if isinstance(args, dict) else 12
        if not isinstance(count, int) or count < 1 or count > 24:
            return {
                "protocol_version": 1,
                "request_id": request_id,
                "ok": False,
                "error": {
                    "code": "invalid_request",
                    "detail": "count must be an integer between 1 and 24",
                },
            }

        # Return up to requested count of fake reels (tiling fake reels if needed)
        items = []
        for i in range(count):
            base_item = FAKE_REELS[i % len(FAKE_REELS)]
            if i < len(FAKE_REELS):
                items.append(base_item)
            else:
                item_copy = dict(base_item)
                item_copy["id"] = f"{base_item['id']}_{i}"
                items.append(item_copy)

        return {
            "protocol_version": 1,
            "request_id": request_id,
            "ok": True,
            "result": {
                "items": items,
            },
        }

    return {
        "protocol_version": 1,
        "request_id": request_id,
        "ok": False,
        "error": {
            "code": "unknown_command",
            "detail": f"Unknown command: {cmd}",
        },
    }


def serve(
    socket_path: str, simulated_error: Optional[str] = None, run_once: bool = False
) -> None:
    """Run the UDS fake gateway server."""
    sock_path = pathlib.Path(socket_path)
    if sock_path.exists():
        sock_path.unlink()

    sock_path.parent.mkdir(parents=True, exist_ok=True)

    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(str(sock_path))
    server.listen(5)

    def cleanup(*_: Any) -> None:
        server.close()
        if sock_path.exists():
            sock_path.unlink()
        sys.exit(0)

    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)

    try:
        while True:
            conn, _ = server.accept()
            with conn:
                try:
                    # Read 4-byte header
                    header = conn.recv(4)
                    if not header or len(header) < 4:
                        continue
                    payload_len = int.from_bytes(header, byteorder="big")

                    # Read payload_len bytes
                    data = bytearray()
                    while len(data) < payload_len:
                        chunk = conn.recv(payload_len - len(data))
                        if not chunk:
                            break
                        data.extend(chunk)

                    full_frame = header + bytes(data)
                    req_dict = decode_frame(full_frame)
                    resp_dict = handle_request(
                        req_dict, simulated_error=simulated_error
                    )
                    resp_frame = encode_frame(resp_dict)
                    conn.sendall(resp_frame)
                except Exception as exc:
                    err_resp = {
                        "protocol_version": 1,
                        "request_id": "err",
                        "ok": False,
                        "error": {
                            "code": "internal_error",
                            "detail": str(exc),
                        },
                    }
                    try:
                        conn.sendall(encode_frame(err_resp))
                    except Exception:
                        pass
            if run_once:
                break
    finally:
        server.close()
        if sock_path.exists():
            sock_path.unlink()


def main() -> None:
    parser = argparse.ArgumentParser(description="IG-Reels-CLI Fake Gateway Server")
    parser.add_argument(
        "--socket-path",
        default=os.environ.get("IG_REELS_SOCKET_PATH", DEFAULT_SOCKET_PATH),
        help="Path to the Unix domain socket",
    )
    parser.add_argument(
        "--error-code",
        default=os.environ.get("FAKE_GATEWAY_ERROR"),
        help="Simulate a specific error code for get_reels (e.g. auth_required)",
    )
    parser.add_argument(
        "--once",
        action="store_true",
        help="Exit after handling a single connection request",
    )
    args = parser.parse_args()

    serve(
        socket_path=args.socket_path,
        simulated_error=args.error_code,
        run_once=args.once,
    )


if __name__ == "__main__":
    main()
