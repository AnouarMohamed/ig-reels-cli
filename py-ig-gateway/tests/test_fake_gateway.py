import importlib.util
import os
import pathlib
import socket
import sys
import threading

from protocol import decode_frame, encode_frame

# Load fake-gateway.py module dynamically since filename contains a hyphen
REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
GATEWAY_PATH = REPO_ROOT / "scripts" / "fake-gateway.py"

spec = importlib.util.spec_from_file_location("fake_gateway", GATEWAY_PATH)
fake_gateway = importlib.util.module_from_spec(spec)
sys.modules["fake_gateway"] = fake_gateway
spec.loader.exec_module(fake_gateway)

FAKE_REELS = fake_gateway.FAKE_REELS
handle_request = fake_gateway.handle_request
serve = fake_gateway.serve


def test_handle_request_ping():
    req = {
        "protocol_version": 1,
        "request_id": "r-ping",
        "cmd": "ping",
        "args": {},
    }
    resp = handle_request(req)
    assert resp["ok"] is True
    assert resp["request_id"] == "r-ping"
    assert resp["result"]["service"] == "ig-gateway"
    assert resp["result"]["status"] == "ok"


def test_handle_request_get_reels():
    req = {
        "protocol_version": 1,
        "request_id": "r-reels",
        "cmd": "get_reels",
        "args": {"count": 2},
    }
    resp = handle_request(req)
    assert resp["ok"] is True
    assert resp["request_id"] == "r-reels"
    items = resp["result"]["items"]
    assert len(items) == 2
    for item in items:
        assert item["video_url"].startswith("https://example.com/")
        assert item["id"]
        assert item["username"]


def test_handle_request_simulated_error():
    req = {
        "protocol_version": 1,
        "request_id": "r-err",
        "cmd": "get_reels",
        "args": {"count": 5},
    }
    resp = handle_request(req, simulated_error="auth_required")
    assert resp["ok"] is False
    assert resp["request_id"] == "r-err"
    assert resp["error"]["code"] == "auth_required"


def test_handle_request_invalid_count():
    req = {
        "protocol_version": 1,
        "request_id": "r-inv",
        "cmd": "get_reels",
        "args": {"count": 99},
    }
    resp = handle_request(req)
    assert resp["ok"] is False
    assert resp["error"]["code"] == "invalid_request"


def test_handle_request_unknown_cmd():
    req = {
        "protocol_version": 1,
        "request_id": "r-unk",
        "cmd": "nonexistent_command",
        "args": {},
    }
    resp = handle_request(req)
    assert resp["ok"] is False
    assert resp["error"]["code"] == "unknown_command"


def test_uds_server_ping_and_get_reels(tmp_path):
    sock_path = str(tmp_path / "test_fake.sock")

    def client_ping_and_reels():
        # Ping connection
        client1 = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client1.connect(sock_path)
        ping_req = encode_frame(
            {
                "protocol_version": 1,
                "request_id": "req-1",
                "cmd": "ping",
                "args": {},
            }
        )
        client1.sendall(ping_req)

        # Read ping response
        hdr1 = client1.recv(4)
        len1 = int.from_bytes(hdr1, "big")
        payload1 = client1.recv(len1)
        resp1 = decode_frame(hdr1 + payload1)
        assert resp1["ok"] is True
        assert resp1["result"]["service"] == "ig-gateway"
        client1.close()

        # Get reels connection
        client2 = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client2.connect(sock_path)
        reels_req = encode_frame(
            {
                "protocol_version": 1,
                "request_id": "req-2",
                "cmd": "get_reels",
                "args": {"count": 3},
            }
        )
        client2.sendall(reels_req)

        # Read reels response
        hdr2 = client2.recv(4)
        len2 = int.from_bytes(hdr2, "big")
        payload2 = client2.recv(len2)
        resp2 = decode_frame(hdr2 + payload2)
        assert resp2["ok"] is True
        items = resp2["result"]["items"]
        assert len(items) == 3
        assert items[0]["id"] == FAKE_REELS[0]["id"]
        assert items[0]["video_url"].startswith("https://example.com/")
        client2.close()

    # Start server thread
    server_thread = threading.Thread(
        target=serve,
        kwargs={"socket_path": sock_path, "run_once": False},
        daemon=True,
    )
    server_thread.start()

    # Wait for socket to exist
    import time

    for _ in range(50):
        if os.path.exists(sock_path):
            break
        time.sleep(0.05)

    assert os.path.exists(sock_path)

    # Run client checks
    client_ping_and_reels()
