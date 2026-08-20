import sys
import struct
import pytest

# Add the py-ig-gateway directory to the path so we can import protocol
sys.path.insert(0, "/home/anouar/ig-reels-cli/py-ig-gateway")
from protocol import (
    encode_frame,
    decode_frame,
    ProtocolError,
    InvalidPayloadLength,
    IncompleteFrame,
    MAX_PAYLOAD,
)


def test_encode_decode_roundtrip():
    msg = {"protocol_version": 1, "request_id": "r1", "cmd": "ping", "args": {}}
    frame = encode_frame(msg)
    decoded = decode_frame(frame)
    assert decoded == msg


def test_encode_decode_with_payload():
    msg = {
        "protocol_version": 1,
        "request_id": "r2",
        "cmd": "get_reels",
        "args": {"count": 12},
    }
    frame = encode_frame(msg)
    decoded = decode_frame(frame)
    assert decoded == msg


def test_zero_length_payload():
    frame = struct.pack(">I", 0)  # zero length
    with pytest.raises(InvalidPayloadLength):
        decode_frame(frame)


def test_oversize_payload():
    large_str = "x" * (MAX_PAYLOAD + 100)
    msg = {"data": large_str}
    with pytest.raises(InvalidPayloadLength):
        encode_frame(msg)


def test_truncated_payload():
    msg = {"test": "truncated"}
    frame = encode_frame(msg)
    truncated_frame = frame[:-1]
    with pytest.raises(IncompleteFrame):
        decode_frame(truncated_frame)


def test_header_too_short():
    frame = b"\x00\x00\x00"
    with pytest.raises(ProtocolError):
        decode_frame(frame)


def test_non_dict_input():
    # We assume the caller passes a dict; we don't test non-dict.
    pass


def test_extra_bytes_after_payload():
    msg = {"test": "extra"}
    frame = encode_frame(msg)
    frame_with_extra = frame + b"\x00"
    with pytest.raises(ProtocolError):
        decode_frame(frame_with_extra)


def test_invalid_msgpack():
    header = struct.pack(">I", 2)
    payload = b"\xff\xff"
    frame = header + payload
    with pytest.raises(Exception):
        decode_frame(frame)


def test_max_payload():
    big_str_under = "x" * (MAX_PAYLOAD - 100)
    msg_under = {"data": big_str_under}
    frame_under = encode_frame(msg_under)
    decoded_under = decode_frame(frame_under)
    assert decoded_under == msg_under

    big_str_over = "x" * (MAX_PAYLOAD + 100)
    msg_over = {"data": big_str_over}
    with pytest.raises(InvalidPayloadLength):
        encode_frame(msg_over)
