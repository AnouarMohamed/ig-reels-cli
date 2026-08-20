import struct
import msgpack

MAX_PAYLOAD = 1_048_576  # 1 MiB


class ProtocolError(ValueError):
    """Base class for protocol errors."""

    pass


class InvalidPayloadLength(ProtocolError):
    """Raised when payload length is zero or exceeds MAX_PAYLOAD."""

    pass


class IncompleteFrame(ProtocolError):
    """Raised when the frame buffer is too short for the declared payload."""

    pass


def encode_frame(msg: dict) -> bytes:
    """
    Encode a dict into a framed MessagePack message.

    Args:
        msg: The dictionary to encode.

    Returns:
        bytes: The framed message (4-byte length + MessagePack data).

    Raises:
        InvalidPayloadLength: If the encoded payload length is zero or exceeds MAX_PAYLOAD.
    """
    payload = msgpack.packb(msg, use_bin_type=True)
    length = len(payload)
    if length == 0 or length > MAX_PAYLOAD:
        raise InvalidPayloadLength(
            f"Payload length {length} is invalid (must be 1-{MAX_PAYLOAD})"
        )
    # Pack length as 4-byte big-endian unsigned int
    header = struct.pack(">I", length)
    return header + payload


def decode_frame(frame: bytes) -> dict:
    """
    Decode a framed MessagePack message into a dict.

    Args:
        frame: The framed message (must include 4-byte header and payload).

    Returns:
        dict: The decoded MessagePack object.

    Raises:
        ProtocolError: If the frame is too short for the header or if there are extra bytes.
        InvalidPayloadLength: If the payload length is zero or exceeds MAX_PAYLOAD.
        IncompleteFrame: If the frame buffer is too short for the declared payload.
        msgpack.exceptions.ExtraData: If there are extra bytes after the payload in the MessagePack data.
        msgpack.exceptions.UnpackException: If the payload is not valid MessagePack.
    """
    if len(frame) < 4:
        raise ProtocolError("Frame too short for header (less than 4 bytes)")

    (length,) = struct.unpack(">I", frame[:4])
    if length == 0 or length > MAX_PAYLOAD:
        raise InvalidPayloadLength(
            f"Payload length {length} is invalid (must be 1-{MAX_PAYLOAD})"
        )

    expected_len = 4 + length
    if len(frame) < expected_len:
        raise IncompleteFrame(
            f"Frame too short: expected {expected_len} bytes, got {len(frame)}"
        )
    if len(frame) > expected_len:
        raise ProtocolError(
            f"Frame too long: expected {expected_len} bytes, got {len(frame)}"
        )
    # Extract payload
    payload = frame[4:expected_len]
    return msgpack.unpackb(payload, raw=False, strict_map_key=True)
