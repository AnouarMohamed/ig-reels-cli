use rust_tui::ipc::{read_frame, write_frame, Request};
use std::io::Cursor;

#[tokio::test]
async fn test_rust_decodes_named_map_bytes() {
    let map_val = serde_json::json!({
        "protocol_version": 1,
        "request_id": "req-py-cross",
        "cmd": "ping",
        "args": {}
    });

    let payload = rmp_serde::to_vec_named(&map_val).expect("serialize named map");

    let mut buf = Vec::new();
    write_frame(&mut buf, &payload).await.expect("write_frame");

    let mut cursor = Cursor::new(buf);
    let frame_payload = read_frame(&mut cursor).await.expect("read_frame");

    let req: Request = rmp_serde::from_slice(&frame_payload).expect("rmp_serde decode");
    assert_eq!(req.protocol_version, 1);
    assert_eq!(req.request_id, "req-py-cross");
    assert_eq!(req.cmd, "ping");
}

#[test]
fn test_rust_named_map_encoding_produces_map_payload() {
    let req = Request::ping("req-named");
    let bytes = rmp_serde::to_vec_named(&req).expect("to_vec_named");

    // The first byte of MessagePack map with 4 elements is 0x84 (fixmap with 4 pairs)
    assert!(
        (0x80..=0x8f).contains(&bytes[0]) || bytes[0] == 0xde || bytes[0] == 0xdf,
        "rmp_serde::to_vec_named must serialize struct as MessagePack map (not array/tuple)"
    );
}

#[test]
fn test_rust_compact_tuple_encoding_produces_array_payload() {
    let req = Request::ping("req-tuple");
    let bytes = rmp_serde::to_vec(&req).expect("to_vec");

    // The first byte of MessagePack array with 4 elements is 0x94 (fixarray with 4 elements)
    assert!(
        (0x90..=0x9f).contains(&bytes[0]) || bytes[0] == 0xdc || bytes[0] == 0xdd,
        "rmp_serde::to_vec serializes struct as MessagePack array/tuple"
    );
}
