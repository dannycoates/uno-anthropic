use serde::{Deserialize, Serialize};

/// Metadata attached to a message request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Cache control directive for content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: String,
}

impl CacheControl {
    /// Create a new ephemeral cache control directive.
    pub fn ephemeral() -> Self {
        Self {
            cache_type: "ephemeral".to_string(),
        }
    }
}

/// Service tier for request routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Auto,
    StandardOnly,
}

/// Output configuration for message responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<JsonOutputFormat>,
}

/// JSON output format configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutputFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

impl JsonOutputFormat {
    /// Create a JSON output format.
    pub fn json() -> Self {
        Self {
            format_type: "json".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_serialize_empty() {
        let meta = Metadata::default();
        let json = serde_json::to_string(&meta).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_metadata_serialize_with_user_id() {
        let meta = Metadata {
            user_id: Some("user-123".to_string()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert_eq!(json, r#"{"user_id":"user-123"}"#);
    }

    #[test]
    fn test_cache_control_ephemeral() {
        let cc = CacheControl::ephemeral();
        let json = serde_json::to_string(&cc).unwrap();
        assert_eq!(json, r#"{"type":"ephemeral"}"#);
    }

    #[test]
    fn test_service_tier_roundtrip() {
        let auto_json = serde_json::to_string(&ServiceTier::Auto).unwrap();
        assert_eq!(auto_json, r#""auto""#);
        let auto: ServiceTier = serde_json::from_str(&auto_json).unwrap();
        assert_eq!(auto, ServiceTier::Auto);

        let standard_json = serde_json::to_string(&ServiceTier::StandardOnly).unwrap();
        assert_eq!(standard_json, r#""standard_only""#);
    }

    #[test]
    fn test_json_output_format() {
        let fmt = JsonOutputFormat::json();
        let json = serde_json::to_string(&fmt).unwrap();
        assert_eq!(json, r#"{"type":"json"}"#);
    }

    #[test]
    fn test_output_config_serialize() {
        let config = OutputConfig {
            format: Some(JsonOutputFormat::json()),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"format":{"type":"json"}}"#);
    }

    #[test]
    fn test_output_config_serialize_none() {
        let config = OutputConfig { format: None };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{}"#);
    }
}
