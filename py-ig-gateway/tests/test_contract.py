import pathlib
import sys
import msgpack

# Add the py-ig-gateway directory to sys.path
GATEWAY_DIR = pathlib.Path(__file__).resolve().parent.parent
if str(GATEWAY_DIR) not in sys.path:
    sys.path.insert(0, str(GATEWAY_DIR))

from protocol import decode_frame, encode_frame  # noqa: E402


def test_python_encode_maps_with_string_keys():
    req = {
        "protocol_version": 1,
        "request_id": "req-py-1",
        "cmd": "get_reels",
        "args": {"count": 10},
    }
    frame = encode_frame(req)
    decoded = decode_frame(frame)
    assert decoded["protocol_version"] == 1
    assert decoded["request_id"] == "req-py-1"
    assert decoded["cmd"] == "get_reels"
    assert decoded["args"] == {"count": 10}


def test_rust_named_map_decodes_as_dict_in_python():
    """Verify Rust named-map encoding decodes as dict in Python."""
    rust_named_map = msgpack.packb(
        {
            "protocol_version": 1,
            "request_id": "r-rust-named",
            "cmd": "ping",
            "args": {},
        },
        use_bin_type=True,
    )
    header = len(rust_named_map).to_bytes(4, byteorder="big")
    frame = header + rust_named_map

    decoded = decode_frame(frame)
    assert isinstance(decoded, dict), (
        "Rust named-map encoding must decode as a Python dict"
    )
    assert decoded["protocol_version"] == 1
    assert decoded["request_id"] == "r-rust-named"


def test_rust_compact_tuple_encoding_fails_map_contract():
    """Verify that compact tuple (array) encoding fails map contract expectation."""
    # A MessagePack array (tuple encoding of a struct) instead of a map
    rust_tuple_bytes = msgpack.packb([1, "r-rust-tuple", "ping", {}], use_bin_type=True)
    header = len(rust_tuple_bytes).to_bytes(4, byteorder="big")
    frame = header + rust_tuple_bytes

    decoded = decode_frame(frame)
    # The protocol contract requires map with string keys. A tuple encoding returns a list.
    assert not isinstance(decoded, dict), (
        "Tuple encoding must fail dictionary map expectation"
    )
