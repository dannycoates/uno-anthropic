use serde::{Deserialize, Serialize};

/// Known Anthropic model identifiers.
///
/// Use one of the known variants for type safety, or `Model::Other(String)`
/// for model IDs not yet represented here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Model {
    #[serde(rename = "claude-opus-4-6")]
    ClaudeOpus4_6,
    #[serde(rename = "claude-sonnet-4-6")]
    ClaudeSonnet4_6,
    #[serde(rename = "claude-opus-4-5-20251101")]
    ClaudeOpus4_5_20251101,
    #[serde(rename = "claude-opus-4-5")]
    ClaudeOpus4_5,
    #[serde(rename = "claude-opus-4-1-20250805")]
    ClaudeOpus4_1_20250805,
    #[serde(rename = "claude-opus-4-0")]
    ClaudeOpus4_0,
    #[serde(rename = "claude-opus-4-20250514")]
    ClaudeOpus4_20250514,
    #[serde(rename = "claude-4-opus-20250514")]
    Claude4Opus20250514,
    #[serde(rename = "claude-sonnet-4-5")]
    ClaudeSonnet4_5,
    #[serde(rename = "claude-sonnet-4-5-20250929")]
    ClaudeSonnet4_5_20250929,
    #[serde(rename = "claude-sonnet-4-0")]
    ClaudeSonnet4_0,
    #[serde(rename = "claude-sonnet-4-20250514")]
    ClaudeSonnet4_20250514,
    #[serde(rename = "claude-4-sonnet-20250514")]
    Claude4Sonnet20250514,
    #[serde(rename = "claude-haiku-4-5")]
    ClaudeHaiku4_5,
    #[serde(rename = "claude-haiku-4-5-20251001")]
    ClaudeHaiku4_5_20251001,
    #[serde(rename = "claude-3-7-sonnet-latest")]
    Claude3_7SonnetLatest,
    #[serde(rename = "claude-3-7-sonnet-20250219")]
    Claude3_7Sonnet20250219,
    #[serde(rename = "claude-3-5-haiku-latest")]
    Claude3_5HaikuLatest,
    #[serde(rename = "claude-3-5-haiku-20241022")]
    Claude3_5Haiku20241022,
    #[serde(rename = "claude-3-opus-latest")]
    Claude3OpusLatest,
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus20240229,
    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku20240307,
    /// Any model ID not in the known variants.
    #[serde(untagged)]
    Other(String),
}

impl<S: Into<String>> From<S> for Model {
    fn from(s: S) -> Self {
        let s = s.into();
        // Try to deserialize as a known variant first
        match serde_json::from_value::<Model>(serde_json::Value::String(s.clone())) {
            Ok(model) => model,
            Err(_) => Model::Other(s),
        }
    }
}

impl Model {
    /// Parse a model from a string, falling back to `Model::Other(s.to_string())`
    /// for unknown model IDs.
    ///
    /// Returns whether this model supports extended thinking.
    ///
    /// Models that don't support thinking will have thinking config stripped
    /// before API calls to prevent `invalid_request_error` responses.
    pub fn supports_extended_thinking(&self) -> bool {
        match self {
            // Claude 4.x Opus and Sonnet support extended thinking
            Model::ClaudeOpus4_6
            | Model::ClaudeOpus4_5_20251101
            | Model::ClaudeOpus4_5
            | Model::ClaudeOpus4_1_20250805
            | Model::ClaudeOpus4_0
            | Model::ClaudeOpus4_20250514
            | Model::Claude4Opus20250514
            | Model::ClaudeSonnet4_6
            | Model::ClaudeSonnet4_5
            | Model::ClaudeSonnet4_5_20250929
            | Model::ClaudeSonnet4_0
            | Model::ClaudeSonnet4_20250514
            | Model::Claude4Sonnet20250514
            // Claude 3.7 Sonnet supports extended thinking
            | Model::Claude3_7SonnetLatest
            | Model::Claude3_7Sonnet20250219 => true,
            // Haiku and older Claude 3.x models don't support extended thinking
            Model::ClaudeHaiku4_5
            | Model::ClaudeHaiku4_5_20251001
            | Model::Claude3_5HaikuLatest
            | Model::Claude3_5Haiku20241022
            | Model::Claude3OpusLatest
            | Model::Claude3Opus20240229
            | Model::Claude3Haiku20240307 => false,
            // Unknown models: allow optimistically (API will reject if unsupported)
            Model::Other(_) => true,
        }
    }

    /// Short aliases are resolved before parsing:
    /// - `"sonnet"` → `"claude-sonnet-4-6"`
    /// - `"opus"`   → `"claude-opus-4-6"`
    /// - `"haiku"`  → `"claude-haiku-4-5"`
    pub fn from_str_lossy(s: &str) -> Self {
        let resolved = match s {
            "sonnet" => "claude-sonnet-4-6",
            "opus"   => "claude-opus-4-6",
            "haiku"  => "claude-haiku-4-5",
            other    => other,
        };
        match serde_json::from_value::<Model>(serde_json::Value::String(resolved.to_string())) {
            Ok(model) => model,
            Err(_) => Model::Other(s.to_string()),
        }
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::Other(s) => f.write_str(s),
            _ => {
                // Serialize to get the serde rename value
                let val = serde_json::to_value(self).unwrap_or_default();
                if let serde_json::Value::String(s) = val {
                    f.write_str(&s)
                } else {
                    write!(f, "{:?}", self)
                }
            }
        }
    }
}

/// Information about a model returned by the Models API.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub model_type: String,
    pub display_name: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A parsed model specification that may include option flags.
///
/// Parses strings like `"sonnet"`, `"claude-sonnet-4-6"`, `"sonnet[1m]"`, `"opus[1m]"`.
/// The `[1m]` suffix enables the 1M-token extended context window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSpec {
    pub model: Model,
    pub extended_context: bool,
}

impl ModelSpec {
    pub fn parse(s: &str) -> Self {
        let (base, extended_context) = if let Some(b) = s.strip_suffix("[1m]") {
            (b, true)
        } else {
            (s, false)
        };
        ModelSpec {
            model: Model::from_str_lossy(base),
            extended_context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_known_model() {
        let model = Model::ClaudeOpus4_6;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, r#""claude-opus-4-6""#);
    }

    #[test]
    fn test_deserialize_known_model() {
        let model: Model = serde_json::from_str(r#""claude-opus-4-6""#).unwrap();
        assert_eq!(model, Model::ClaudeOpus4_6);
    }

    #[test]
    fn test_deserialize_unknown_model() {
        let model: Model = serde_json::from_str(r#""claude-future-model""#).unwrap();
        assert_eq!(model, Model::Other("claude-future-model".to_string()));
    }

    #[test]
    fn test_serialize_other_model() {
        let model = Model::Other("custom-model".to_string());
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, r#""custom-model""#);
    }

    #[test]
    fn test_roundtrip_all_known_variants() {
        let variants = vec![
            (Model::ClaudeOpus4_6, "claude-opus-4-6"),
            (Model::ClaudeSonnet4_6, "claude-sonnet-4-6"),
            (Model::ClaudeOpus4_5_20251101, "claude-opus-4-5-20251101"),
            (Model::ClaudeOpus4_5, "claude-opus-4-5"),
            (Model::ClaudeOpus4_1_20250805, "claude-opus-4-1-20250805"),
            (Model::ClaudeOpus4_0, "claude-opus-4-0"),
            (Model::ClaudeOpus4_20250514, "claude-opus-4-20250514"),
            (Model::Claude4Opus20250514, "claude-4-opus-20250514"),
            (Model::ClaudeSonnet4_5, "claude-sonnet-4-5"),
            (Model::ClaudeSonnet4_5_20250929, "claude-sonnet-4-5-20250929"),
            (Model::ClaudeSonnet4_0, "claude-sonnet-4-0"),
            (Model::ClaudeSonnet4_20250514, "claude-sonnet-4-20250514"),
            (Model::Claude4Sonnet20250514, "claude-4-sonnet-20250514"),
            (Model::ClaudeHaiku4_5, "claude-haiku-4-5"),
            (Model::ClaudeHaiku4_5_20251001, "claude-haiku-4-5-20251001"),
            (Model::Claude3_7SonnetLatest, "claude-3-7-sonnet-latest"),
            (Model::Claude3_7Sonnet20250219, "claude-3-7-sonnet-20250219"),
            (Model::Claude3_5HaikuLatest, "claude-3-5-haiku-latest"),
            (Model::Claude3_5Haiku20241022, "claude-3-5-haiku-20241022"),
            (Model::Claude3OpusLatest, "claude-3-opus-latest"),
            (Model::Claude3Opus20240229, "claude-3-opus-20240229"),
            (Model::Claude3Haiku20240307, "claude-3-haiku-20240307"),
        ];

        for (variant, expected_str) in variants {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str));

            let deserialized: Model = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(Model::ClaudeOpus4_6.to_string(), "claude-opus-4-6");
        assert_eq!(
            Model::Other("my-model".to_string()).to_string(),
            "my-model"
        );
    }

    #[test]
    fn test_deserialize_model_info() {
        let json = r#"{
            "id": "claude-opus-4-6",
            "type": "model",
            "display_name": "Claude Opus 4.6",
            "created_at": "2025-01-01T00:00:00Z"
        }"#;
        let info: ModelInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, "claude-opus-4-6");
        assert_eq!(info.model_type, "model");
        assert_eq!(info.display_name, "Claude Opus 4.6");
        assert_eq!(info.created_at.as_deref(), Some("2025-01-01T00:00:00Z"));
    }

    #[test]
    fn test_alias_sonnet() {
        assert_eq!(Model::from_str_lossy("sonnet"), Model::ClaudeSonnet4_6);
    }

    #[test]
    fn test_alias_opus() {
        assert_eq!(Model::from_str_lossy("opus"), Model::ClaudeOpus4_6);
    }

    #[test]
    fn test_alias_haiku() {
        assert_eq!(Model::from_str_lossy("haiku"), Model::ClaudeHaiku4_5);
    }

    #[test]
    fn test_supports_extended_thinking_opus() {
        assert!(Model::ClaudeOpus4_6.supports_extended_thinking());
        assert!(Model::ClaudeOpus4_5.supports_extended_thinking());
        assert!(Model::ClaudeOpus4_0.supports_extended_thinking());
    }

    #[test]
    fn test_supports_extended_thinking_sonnet() {
        assert!(Model::ClaudeSonnet4_6.supports_extended_thinking());
        assert!(Model::ClaudeSonnet4_5.supports_extended_thinking());
        assert!(Model::ClaudeSonnet4_0.supports_extended_thinking());
    }

    #[test]
    fn test_supports_extended_thinking_3_7_sonnet() {
        assert!(Model::Claude3_7SonnetLatest.supports_extended_thinking());
        assert!(Model::Claude3_7Sonnet20250219.supports_extended_thinking());
    }

    #[test]
    fn test_no_extended_thinking_haiku() {
        assert!(!Model::ClaudeHaiku4_5.supports_extended_thinking());
        assert!(!Model::ClaudeHaiku4_5_20251001.supports_extended_thinking());
        assert!(!Model::Claude3_5HaikuLatest.supports_extended_thinking());
        assert!(!Model::Claude3_5Haiku20241022.supports_extended_thinking());
        assert!(!Model::Claude3Haiku20240307.supports_extended_thinking());
    }

    #[test]
    fn test_no_extended_thinking_claude3_opus() {
        assert!(!Model::Claude3OpusLatest.supports_extended_thinking());
        assert!(!Model::Claude3Opus20240229.supports_extended_thinking());
    }

    #[test]
    fn test_extended_thinking_unknown_model_optimistic() {
        assert!(Model::Other("future-model".to_string()).supports_extended_thinking());
    }

    #[test]
    fn test_model_spec_parse_plain() {
        let spec = ModelSpec::parse("sonnet");
        assert_eq!(spec.model, Model::ClaudeSonnet4_6);
        assert!(!spec.extended_context);
    }

    #[test]
    fn test_model_spec_parse_extended() {
        let spec = ModelSpec::parse("sonnet[1m]");
        assert_eq!(spec.model, Model::ClaudeSonnet4_6);
        assert!(spec.extended_context);
    }

    #[test]
    fn test_model_spec_parse_full_id_extended() {
        let spec = ModelSpec::parse("claude-opus-4-6[1m]");
        assert_eq!(spec.model, Model::ClaudeOpus4_6);
        assert!(spec.extended_context);
    }
}
