use serde::{Deserialize, Serialize};
use std::fmt;

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_REQUEST_ID_LEN: usize = 64;
pub const MAX_REEL_ID_LEN: usize = 128;
pub const MAX_VIDEO_URL_LEN: usize = 8192;
pub const MAX_CAPTION_LEN: usize = 20000;
pub const MAX_USERNAME_LEN: usize = 256;

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
}
