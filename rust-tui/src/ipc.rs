use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_REQUEST_ID_LEN: usize = 64;
pub const MAX_REEL_ID_LEN: usize = 128;
pub const MAX_VIDEO_URL_LEN: usize = 8192;
pub const MAX_CAPTION_LEN: usize = 20000;
pub const MAX_USERNAME_LEN: usize = 256;
pub const MAX_PAYLOAD_SIZE: usize = 1_048_576; // 1 MiB

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DtoValidationError {
    UnsupportedProtocolVersion(u32),
    InvalidRequestId(String),
    InvalidCommand(String),
    InvalidCount(u32),
    EmptyField(&'static str),
    FieldTooLong {
        field: &'static str,
        len: usize,
        max: usize,
    },
    InvalidUrlScheme(String),
    NonAsciiRequestId(String),
}

impl fmt::Display for DtoValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProtocolVersion(v) => {
                write!(
                    f,
                    "unsupported protocol version: {}, expected {}",
                    v, PROTOCOL_VERSION
                )
            }
            Self::InvalidRequestId(msg) => write!(f, "invalid request_id: {}", msg),
            Self::InvalidCommand(cmd) => write!(f, "invalid command: {}", cmd),
            Self::InvalidCount(cnt) => {
                write!(f, "invalid count: {}, must be between 1 and 24", cnt)
            }
            Self::EmptyField(field) => write!(f, "field '{}' cannot be empty", field),
            Self::FieldTooLong { field, len, max } => {
                write!(f, "field '{}' length {} exceeds max {}", field, len, max)
            }
            Self::InvalidUrlScheme(url) => {
                write!(
                    f,
                    "invalid URL scheme for '{}': must start with https://",
                    url
                )
            }
            Self::NonAsciiRequestId(id) => {
                write!(
                    f,
                    "request_id '{}' contains non-ASCII printable characters",
                    id
                )
            }
        }
    }
}

impl std::error::Error for DtoValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcFrameError {
    InvalidPayloadLength(u32),
    ProtocolTruncated,
    IoError(String),
}

impl fmt::Display for IpcFrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayloadLength(len) => {
                write!(
                    f,
                    "invalid payload length {}, must be 1-{}",
                    len, MAX_PAYLOAD_SIZE
                )
            }
            Self::ProtocolTruncated => write!(f, "protocol stream truncated before frame end"),
            Self::IoError(msg) => write!(f, "IPC I/O error: {}", msg),
        }
    }
}

impl std::error::Error for IpcFrameError {}

#[derive(Debug)]
pub enum IpcClientError {
    ConnectTimeout,
    RequestTimeout,
    IoError(std::io::Error),
    FrameError(IpcFrameError),
    SerializationError(String),
    ValidationError(DtoValidationError),
    RequestIdMismatch { expected: String, actual: String },
}

impl fmt::Display for IpcClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectTimeout => write!(f, "connection to IPC gateway timed out (2s limit)"),
            Self::RequestTimeout => write!(f, "IPC request/response timed out (30s limit)"),
            Self::IoError(e) => write!(f, "IPC I/O error: {}", e),
            Self::FrameError(e) => write!(f, "IPC frame error: {}", e),
            Self::SerializationError(msg) => write!(f, "IPC serialization error: {}", msg),
            Self::ValidationError(e) => write!(f, "IPC DTO validation error: {}", e),
            Self::RequestIdMismatch { expected, actual } => {
                write!(
                    f,
                    "IPC request_id mismatch: expected '{}', got '{}'",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for IpcClientError {}

impl From<std::io::Error> for IpcClientError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<IpcFrameError> for IpcClientError {
    fn from(e: IpcFrameError) -> Self {
        Self::FrameError(e)
    }
}

impl From<DtoValidationError> for IpcClientError {
    fn from(e: DtoValidationError) -> Self {
        Self::ValidationError(e)
    }
}

pub async fn read_frame<R>(reader: &mut R) -> Result<Vec<u8>, IpcFrameError>
where
    R: AsyncReadExt + Unpin,
{
    let mut header = [0u8; 4];
    match reader.read_exact(&mut header).await {
        Ok(4) => {}
        Ok(_) => return Err(IpcFrameError::ProtocolTruncated),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(IpcFrameError::ProtocolTruncated);
        }
        Err(e) => return Err(IpcFrameError::IoError(e.to_string())),
    }

    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > MAX_PAYLOAD_SIZE {
        return Err(IpcFrameError::InvalidPayloadLength(length as u32));
    }

    let mut payload = vec![0u8; length];
    match reader.read_exact(&mut payload).await {
        Ok(_) => Ok(payload),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            Err(IpcFrameError::ProtocolTruncated)
        }
        Err(e) => Err(IpcFrameError::IoError(e.to_string())),
    }
}

pub async fn write_frame<W>(writer: &mut W, payload: &[u8]) -> Result<(), IpcFrameError>
where
    W: AsyncWriteExt + Unpin,
{
    let length = payload.len();
    if length == 0 || length > MAX_PAYLOAD_SIZE {
        return Err(IpcFrameError::InvalidPayloadLength(length as u32));
    }

    let header = (length as u32).to_be_bytes();
    writer
        .write_all(&header)
        .await
        .map_err(|e| IpcFrameError::IoError(e.to_string()))?;
    writer
        .write_all(payload)
        .await
        .map_err(|e| IpcFrameError::IoError(e.to_string()))?;
    writer
        .flush()
        .await
        .map_err(|e| IpcFrameError::IoError(e.to_string()))?;

    Ok(())
}

pub async fn send_request(
    socket_path: impl AsRef<std::path::Path>,
    request: &Request,
) -> Result<Response, IpcClientError> {
    request.validate()?;

    let socket_path = socket_path.as_ref();
    let stream_fut = UnixStream::connect(socket_path);
    let mut stream = match timeout(CONNECT_TIMEOUT, stream_fut).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => return Err(IpcClientError::IoError(e)),
        Err(_) => return Err(IpcClientError::ConnectTimeout),
    };

    let exchange_fut = async {
        let req_bytes = rmp_serde::to_vec_named(request)
            .map_err(|e| IpcClientError::SerializationError(e.to_string()))?;

        write_frame(&mut stream, &req_bytes).await?;
        let resp_bytes = read_frame(&mut stream).await?;

        let response: Response = rmp_serde::from_slice(&resp_bytes)
            .map_err(|e| IpcClientError::SerializationError(e.to_string()))?;

        response.validate()?;

        if response.request_id != request.request_id {
            return Err(IpcClientError::RequestIdMismatch {
                expected: request.request_id.clone(),
                actual: response.request_id,
            });
        }

        let _ = stream.shutdown().await;

        Ok(response)
    };

    match timeout(REQUEST_TIMEOUT, exchange_fut).await {
        Ok(result) => result,
        Err(_) => Err(IpcClientError::RequestTimeout),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub protocol_version: u32,
    pub request_id: String,
    pub cmd: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

impl Request {
    pub fn ping(request_id: impl Into<String>) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id: request_id.into(),
            cmd: "ping".to_string(),
            args: serde_json::json!({}),
        }
    }

    pub fn get_reels(request_id: impl Into<String>, count: u32) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id: request_id.into(),
            cmd: "get_reels".to_string(),
            args: serde_json::json!({ "count": count }),
        }
    }

    pub fn validate(&self) -> Result<(), DtoValidationError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(DtoValidationError::UnsupportedProtocolVersion(
                self.protocol_version,
            ));
        }

        validate_request_id(&self.request_id)?;

        match self.cmd.as_str() {
            "ping" => Ok(()),
            "get_reels" => {
                let count = self
                    .args
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .ok_or(DtoValidationError::InvalidCount(0))? as u32;

                if !(1..=24).contains(&count) {
                    return Err(DtoValidationError::InvalidCount(count));
                }
                Ok(())
            }
            _ => Err(DtoValidationError::InvalidCommand(self.cmd.clone())),
        }
    }
}

fn validate_request_id(id: &str) -> Result<(), DtoValidationError> {
    if id.is_empty() {
        return Err(DtoValidationError::InvalidRequestId(
            "request_id cannot be empty".to_string(),
        ));
    }
    if id.len() > MAX_REQUEST_ID_LEN {
        return Err(DtoValidationError::FieldTooLong {
            field: "request_id",
            len: id.len(),
            max: MAX_REQUEST_ID_LEN,
        });
    }
    if !id.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        return Err(DtoValidationError::NonAsciiRequestId(id.to_string()));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorDetail {
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub protocol_version: u32,
    pub request_id: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}

impl Response {
    pub fn validate(&self) -> Result<(), DtoValidationError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(DtoValidationError::UnsupportedProtocolVersion(
                self.protocol_version,
            ));
        }
        validate_request_id(&self.request_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PingResult {
    pub service: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetReelsResult {
    pub items: Vec<ReelDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReelDto {
    pub id: String,
    pub video_url: String,
    #[serde(default)]
    pub caption: Option<String>,
    pub username: String,
    #[serde(default)]
    pub like_count: Option<u64>,
}

impl ReelDto {
    pub fn validate(&self) -> Result<(), DtoValidationError> {
        if self.id.is_empty() {
            return Err(DtoValidationError::EmptyField("id"));
        }
        let id_len = self.id.chars().count();
        if id_len > MAX_REEL_ID_LEN {
            return Err(DtoValidationError::FieldTooLong {
                field: "id",
                len: id_len,
                max: MAX_REEL_ID_LEN,
            });
        }

        if self.video_url.is_empty() {
            return Err(DtoValidationError::EmptyField("video_url"));
        }
        if self.video_url.len() > MAX_VIDEO_URL_LEN {
            return Err(DtoValidationError::FieldTooLong {
                field: "video_url",
                len: self.video_url.len(),
                max: MAX_VIDEO_URL_LEN,
            });
        }
        if !self.video_url.starts_with("https://") {
            return Err(DtoValidationError::InvalidUrlScheme(self.video_url.clone()));
        }

        if let Some(ref caption) = self.caption {
            let cap_len = caption.chars().count();
            if cap_len > MAX_CAPTION_LEN {
                return Err(DtoValidationError::FieldTooLong {
                    field: "caption",
                    len: cap_len,
                    max: MAX_CAPTION_LEN,
                });
            }
        }

        if self.username.is_empty() {
            return Err(DtoValidationError::EmptyField("username"));
        }
        let user_len = self.username.chars().count();
        if user_len > MAX_USERNAME_LEN {
            return Err(DtoValidationError::FieldTooLong {
                field: "username",
                len: user_len,
                max: MAX_USERNAME_LEN,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tokio::net::UnixListener;

    #[test]
    fn test_request_ping_serialize_deserialize() {
        let req = Request::ping("req-123");
        assert!(req.validate().is_ok());

        let bytes = rmp_serde::to_vec_named(&req).expect("serialize request");
        let decoded: Request = rmp_serde::from_slice(&bytes).expect("deserialize request");

        assert_eq!(req, decoded);
    }

    #[test]
    fn test_request_get_reels_serialize_deserialize() {
        let req = Request::get_reels("req-456", 10);
        assert!(req.validate().is_ok());

        let bytes = rmp_serde::to_vec_named(&req).expect("serialize request");
        let decoded: Request = rmp_serde::from_slice(&bytes).expect("deserialize request");

        assert_eq!(req, decoded);
    }

    #[test]
    fn test_request_validation_invalid_count() {
        let req0 = Request::get_reels("r0", 0);
        assert!(matches!(
            req0.validate(),
            Err(DtoValidationError::InvalidCount(0))
        ));

        let req25 = Request::get_reels("r25", 25);
        assert!(matches!(
            req25.validate(),
            Err(DtoValidationError::InvalidCount(25))
        ));
    }

    #[test]
    fn test_request_validation_invalid_request_id() {
        let req_empty = Request::ping("");
        assert!(matches!(
            req_empty.validate(),
            Err(DtoValidationError::InvalidRequestId(_))
        ));

        let req_long = Request::ping("a".repeat(65));
        assert!(matches!(
            req_long.validate(),
            Err(DtoValidationError::FieldTooLong {
                field: "request_id",
                ..
            })
        ));
    }

    #[test]
    fn test_reel_dto_validation_success() {
        let dto = ReelDto {
            id: "reel_100".to_string(),
            video_url: "https://example.com/video.mp4".to_string(),
            caption: Some("Hello world".to_string()),
            username: "test_user".to_string(),
            like_count: Some(42),
        };
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_reel_dto_validation_non_https_url() {
        let dto = ReelDto {
            id: "reel_100".to_string(),
            video_url: "http://example.com/video.mp4".to_string(),
            caption: None,
            username: "test_user".to_string(),
            like_count: None,
        };
        assert!(matches!(
            dto.validate(),
            Err(DtoValidationError::InvalidUrlScheme(_))
        ));
    }

    #[test]
    fn test_reel_dto_validation_empty_id() {
        let dto = ReelDto {
            id: "".to_string(),
            video_url: "https://example.com/video.mp4".to_string(),
            caption: None,
            username: "user".to_string(),
            like_count: None,
        };
        assert!(matches!(
            dto.validate(),
            Err(DtoValidationError::EmptyField("id"))
        ));
    }

    #[test]
    fn test_response_error_serialize_deserialize() {
        let resp = Response {
            protocol_version: 1,
            request_id: "req-err".to_string(),
            ok: false,
            result: None,
            error: Some(ErrorDetail {
                code: "auth_required".to_string(),
                detail: "Login required".to_string(),
            }),
        };
        assert!(resp.validate().is_ok());

        let bytes = rmp_serde::to_vec_named(&resp).expect("serialize response");
        let decoded: Response = rmp_serde::from_slice(&bytes).expect("deserialize response");

        assert_eq!(resp, decoded);
    }

    #[tokio::test]
    async fn test_write_and_read_frame_roundtrip() {
        let payload = b"hello msgpack frame";
        let mut buffer = Vec::new();

        write_frame(&mut buffer, payload)
            .await
            .expect("write frame");
        assert_eq!(buffer.len(), 4 + payload.len());

        let mut cursor = Cursor::new(buffer);
        let read_payload = read_frame(&mut cursor).await.expect("read frame");
        assert_eq!(read_payload, payload);
    }

    #[tokio::test]
    async fn test_read_frame_zero_payload_len() {
        let header_zero = 0u32.to_be_bytes().to_vec();
        let mut cursor = Cursor::new(header_zero);
        let res = read_frame(&mut cursor).await;
        assert_eq!(res, Err(IpcFrameError::InvalidPayloadLength(0)));
    }

    #[tokio::test]
    async fn test_read_frame_oversize_payload_len() {
        let oversize = (MAX_PAYLOAD_SIZE as u32 + 1).to_be_bytes().to_vec();
        let mut cursor = Cursor::new(oversize);
        let res = read_frame(&mut cursor).await;
        assert_eq!(
            res,
            Err(IpcFrameError::InvalidPayloadLength(
                MAX_PAYLOAD_SIZE as u32 + 1
            ))
        );
    }

    #[tokio::test]
    async fn test_read_frame_partial_header() {
        let partial_header = vec![0u8, 0u8, 0u8];
        let mut cursor = Cursor::new(partial_header);
        let res = read_frame(&mut cursor).await;
        assert_eq!(res, Err(IpcFrameError::ProtocolTruncated));
    }

    #[tokio::test]
    async fn test_read_frame_truncated_payload() {
        let mut data = 10u32.to_be_bytes().to_vec();
        data.extend_from_slice(b"short");
        let mut cursor = Cursor::new(data);
        let res = read_frame(&mut cursor).await;
        assert_eq!(res, Err(IpcFrameError::ProtocolTruncated));
    }

    #[tokio::test]
    async fn test_write_frame_zero_payload() {
        let mut buffer = Vec::new();
        let res = write_frame(&mut buffer, &[]).await;
        assert_eq!(res, Err(IpcFrameError::InvalidPayloadLength(0)));
    }

    #[tokio::test]
    async fn test_write_frame_oversize_payload() {
        let mut buffer = Vec::new();
        let huge_payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let res = write_frame(&mut buffer, &huge_payload).await;
        assert_eq!(
            res,
            Err(IpcFrameError::InvalidPayloadLength(
                (MAX_PAYLOAD_SIZE + 1) as u32
            ))
        );
    }

    #[tokio::test]
    async fn test_send_request_mock_ping() {
        let sock_path = format!("/tmp/test_ipc_ping_{}.sock", std::process::id());
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).expect("bind socket");

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let req_bytes = read_frame(&mut stream).await.expect("read ping req");
                let req: Request = rmp_serde::from_slice(&req_bytes).expect("decode req");

                let resp = Response {
                    protocol_version: 1,
                    request_id: req.request_id,
                    ok: true,
                    result: Some(serde_json::json!({
                        "service": "ig-gateway",
                        "status": "ok"
                    })),
                    error: None,
                };
                let resp_bytes = rmp_serde::to_vec_named(&resp).expect("encode resp");
                write_frame(&mut stream, &resp_bytes)
                    .await
                    .expect("write ping resp");
            }
        });

        let req = Request::ping("req-ping-1");
        let resp = send_request(&sock_path, &req).await.expect("send_request");

        assert!(resp.ok);
        assert_eq!(resp.request_id, "req-ping-1");
        let res = resp.result.expect("ping result");
        assert_eq!(res["service"], "ig-gateway");

        let _ = std::fs::remove_file(&sock_path);
    }

    #[tokio::test]
    async fn test_send_request_id_mismatch() {
        let sock_path = format!("/tmp/test_ipc_mismatch_{}.sock", std::process::id());
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).expect("bind socket");

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let _ = read_frame(&mut stream).await;
                let resp = Response {
                    protocol_version: 1,
                    request_id: "wrong_id".to_string(),
                    ok: true,
                    result: Some(serde_json::json!({})),
                    error: None,
                };
                let resp_bytes = rmp_serde::to_vec_named(&resp).expect("encode resp");
                let _ = write_frame(&mut stream, &resp_bytes).await;
            }
        });

        let req = Request::ping("expected_id");
        let res = send_request(&sock_path, &req).await;

        assert!(matches!(res, Err(IpcClientError::RequestIdMismatch { .. })));

        let _ = std::fs::remove_file(&sock_path);
    }
}
