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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
}

impl CacheControl {
    /// Create a new ephemeral cache control directive.
    pub fn ephemeral() -> Self {
        Self {
            cache_type: "ephemeral".to_string(),
            ttl: None,
        }
    }

    /// Create a new ephemeral cache control directive with a TTL.
    pub fn ephemeral_with_ttl(ttl: impl Into<String>) -> Self {
        Self {
            cache_type: "ephemeral".to_string(),
            ttl: Some(ttl.into()),
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
    pub format: Option<OutputFormat>,
}

/// Output format configuration.
///
/// Supports plain JSON output or structured JSON schema output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputFormat {
    /// Plain JSON output: `{"type": "json"}`.
    Json,
    /// JSON schema output: `{"type": "json_schema", "name": "...", "schema": {...}}`.
    JsonSchema {
        name: String,
        schema: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    /// Plain text output: `{"type": "text"}`.
    Text,
}

/// Backward-compatible alias. Prefer [`OutputFormat`] for new code.
pub type JsonOutputFormat = OutputFormat;

impl OutputFormat {
    /// Create a plain JSON output format.
    pub fn json() -> Self {
        OutputFormat::Json
    }

    /// Create a JSON schema output format.
    pub fn json_schema(
        name: impl Into<String>,
        schema: serde_json::Value,
        strict: Option<bool>,
    ) -> Self {
        OutputFormat::JsonSchema {
            name: name.into(),
            schema,
            strict,
        }
    }

    /// Create a text output format.
    pub fn text() -> Self {
        OutputFormat::Text
    }
}

/// Reasoning effort level for controlling how much effort the model
/// puts into thinking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Max,
}

/// Geographic region configuration for inference routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceGeo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

impl InferenceGeo {
    /// Create a new inference geo with a region.
    pub fn new(region: impl Into<String>) -> Self {
        Self {
            region: Some(region.into()),
        }
    }
}

/// Context management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManagementConfig {
    pub strategies: Vec<ContextManagementStrategy>,
}

/// A context management strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum ContextManagementStrategy {
    /// Clear thinking blocks from context.
    #[serde(rename = "clear_thinking_20251015")]
    ClearThinking20251015,
    /// Clear tool use blocks from context.
    #[serde(rename = "clear_tool_uses_20250919")]
    ClearToolUses20250919,
    /// Compact context via summarization.
    #[serde(rename = "compact_20260112")]
    Compact20260112,
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
    fn test_cache_control_ephemeral_with_ttl() {
        let cc = CacheControl::ephemeral_with_ttl("5m");
        let json = serde_json::to_string(&cc).unwrap();
        assert!(json.contains(r#""type":"ephemeral""#));
        assert!(json.contains(r#""ttl":"5m""#));
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
        let fmt = OutputFormat::json();
        let json = serde_json::to_string(&fmt).unwrap();
        assert_eq!(json, r#"{"type":"json"}"#);
    }

    #[test]
    fn test_json_schema_output_format() {
        let fmt = OutputFormat::json_schema(
            "my_schema",
            serde_json::json!({"type": "object", "properties": {"name": {"type": "string"}}}),
            Some(true),
        );
        let json = serde_json::to_string(&fmt).unwrap();
        assert!(json.contains(r#""type":"json_schema""#));
        assert!(json.contains(r#""name":"my_schema""#));
        assert!(json.contains(r#""strict":true"#));
        let roundtrip: OutputFormat = serde_json::from_str(&json).unwrap();
        match roundtrip {
            OutputFormat::JsonSchema { name, strict, .. } => {
                assert_eq!(name, "my_schema");
                assert_eq!(strict, Some(true));
            }
            _ => panic!("Expected JsonSchema variant"),
        }
    }

    #[test]
    fn test_output_config_serialize() {
        let config = OutputConfig {
            format: Some(OutputFormat::json()),
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

    #[test]
    fn test_reasoning_effort_roundtrip() {
        let efforts = vec![
            (ReasoningEffort::Low, "low"),
            (ReasoningEffort::Medium, "medium"),
            (ReasoningEffort::High, "high"),
            (ReasoningEffort::Max, "max"),
        ];
        for (effort, expected) in efforts {
            let json = serde_json::to_string(&effort).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
            let roundtrip: ReasoningEffort = serde_json::from_str(&json).unwrap();
            assert_eq!(roundtrip, effort);
        }
    }

    #[test]
    fn test_inference_geo_serialize() {
        let geo = InferenceGeo::new("us");
        let json = serde_json::to_string(&geo).unwrap();
        assert_eq!(json, r#"{"region":"us"}"#);
    }

    #[test]
    fn test_context_management_config_serialize() {
        let config = ContextManagementConfig {
            strategies: vec![
                ContextManagementStrategy::ClearThinking20251015,
                ContextManagementStrategy::Compact20260112,
            ],
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""type":"clear_thinking_20251015""#));
        assert!(json.contains(r#""type":"compact_20260112""#));
        let roundtrip: ContextManagementConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.strategies.len(), 2);
    }

    #[test]
    fn test_text_output_format() {
        let fmt = OutputFormat::text();
        let json = serde_json::to_string(&fmt).unwrap();
        assert_eq!(json, r#"{"type":"text"}"#);
    }
}
