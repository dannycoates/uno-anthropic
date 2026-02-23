use serde::Deserialize;

/// Errors returned by the Anthropic SDK.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (status {status}): {body}")]
    Api { status: u16, body: ApiErrorBody },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Request timed out")]
    Timeout,

    #[error("OAuth error: {0}")]
    OAuth(String),
}

/// Wrapper for the `error` field in API error JSON responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorBody,
}

/// The error detail returned in the `error` field of API error responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorBody {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

impl std::fmt::Display for ApiErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl Error {
    /// Returns `true` if this error is retryable based on the HTTP status code
    /// and error type. Retryable statuses: 408, 409, 429, 5xx.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Api { status, .. } => is_retryable_status(*status),
            Error::Http(e) => {
                if e.is_timeout() {
                    return true;
                }
                if let Some(status) = e.status() {
                    is_retryable_status(status.as_u16())
                } else {
                    // Connection errors are retryable
                    e.is_connect()
                }
            }
            Error::Timeout => true,
            _ => false,
        }
    }
}

/// Check if an HTTP status code is retryable.
pub fn is_retryable_status(status: u16) -> bool {
    matches!(status, 408 | 409 | 429) || status >= 500
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_body_display() {
        let body = ApiErrorBody {
            error_type: "invalid_request_error".to_string(),
            message: "Missing required field".to_string(),
        };
        assert_eq!(
            body.to_string(),
            "invalid_request_error: Missing required field"
        );
    }

    #[test]
    fn test_is_retryable_status() {
        assert!(is_retryable_status(408));
        assert!(is_retryable_status(409));
        assert!(is_retryable_status(429));
        assert!(is_retryable_status(500));
        assert!(is_retryable_status(502));
        assert!(is_retryable_status(503));
        assert!(!is_retryable_status(400));
        assert!(!is_retryable_status(401));
        assert!(!is_retryable_status(403));
        assert!(!is_retryable_status(404));
        assert!(!is_retryable_status(200));
    }

    #[test]
    fn test_api_error_is_retryable() {
        let err = Error::Api {
            status: 429,
            body: ApiErrorBody {
                error_type: "rate_limit_error".to_string(),
                message: "Rate limited".to_string(),
            },
        };
        assert!(err.is_retryable());

        let err = Error::Api {
            status: 400,
            body: ApiErrorBody {
                error_type: "invalid_request_error".to_string(),
                message: "Bad request".to_string(),
            },
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_timeout_is_retryable() {
        let err = Error::Timeout;
        assert!(err.is_retryable());
    }

    #[test]
    fn test_stream_error_not_retryable() {
        let err = Error::StreamError("connection lost".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_deserialize_api_error_body() {
        let json = r#"{"type": "invalid_request_error", "message": "Missing required field"}"#;
        let body: ApiErrorBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.error_type, "invalid_request_error");
        assert_eq!(body.message, "Missing required field");
    }

    #[test]
    fn test_deserialize_api_error_response() {
        let json =
            r#"{"error": {"type": "invalid_request_error", "message": "Missing required field"}}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.error.error_type, "invalid_request_error");
        assert_eq!(resp.error.message, "Missing required field");
    }
}
