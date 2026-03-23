use serde::{Deserialize, Serialize};

/// Configuration for extended thinking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingConfig {
    /// Enable extended thinking with a token budget.
    Enabled {
        budget_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
    /// Disable extended thinking.
    Disabled,
    /// Adaptive thinking (no budget_tokens field).
    Adaptive {
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
}

/// How thinking blocks should be displayed in the response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum ThinkingDisplay {
    /// Return a summarized version of the thinking.
    Summarized,
    /// Omit thinking blocks entirely from the response.
    Omitted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_enabled() {
        let config = ThinkingConfig::Enabled {
            budget_tokens: 10000,
            display: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"enabled","budget_tokens":10000}"#);
    }

    #[test]
    fn test_serialize_disabled() {
        let config = ThinkingConfig::Disabled;
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"disabled"}"#);
    }

    #[test]
    fn test_serialize_adaptive() {
        let config = ThinkingConfig::Adaptive { display: None };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"adaptive"}"#);
    }

    #[test]
    fn test_deserialize_enabled() {
        let json = r#"{"type":"enabled","budget_tokens":5000}"#;
        let config: ThinkingConfig = serde_json::from_str(json).unwrap();
        match config {
            ThinkingConfig::Enabled { budget_tokens, .. } => assert_eq!(budget_tokens, 5000),
            _ => panic!("Expected Enabled variant"),
        }
    }

    #[test]
    fn test_deserialize_disabled() {
        let json = r#"{"type":"disabled"}"#;
        let config: ThinkingConfig = serde_json::from_str(json).unwrap();
        assert!(matches!(config, ThinkingConfig::Disabled));
    }

    #[test]
    fn test_deserialize_adaptive() {
        let json = r#"{"type":"adaptive"}"#;
        let config: ThinkingConfig = serde_json::from_str(json).unwrap();
        assert!(matches!(config, ThinkingConfig::Adaptive { .. }));
    }

    #[test]
    fn test_roundtrip_all_variants() {
        let variants = vec![
            ThinkingConfig::Enabled {
                budget_tokens: 8192,
                display: None,
            },
            ThinkingConfig::Disabled,
            ThinkingConfig::Adaptive { display: None },
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: ThinkingConfig = serde_json::from_str(&json).unwrap();
            // Compare serialized forms since ThinkingConfig doesn't derive PartialEq
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn test_thinking_display_summarized() {
        let config = ThinkingConfig::Enabled {
            budget_tokens: 10000,
            display: Some(ThinkingDisplay::Summarized),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""display":"summarized""#));
        let roundtrip: ThinkingConfig = serde_json::from_str(&json).unwrap();
        match roundtrip {
            ThinkingConfig::Enabled { display, .. } => {
                assert_eq!(display, Some(ThinkingDisplay::Summarized));
            }
            _ => panic!("Expected Enabled variant"),
        }
    }

    #[test]
    fn test_thinking_display_omitted() {
        let config = ThinkingConfig::Enabled {
            budget_tokens: 5000,
            display: Some(ThinkingDisplay::Omitted),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""display":"omitted""#));
    }

    #[test]
    fn test_adaptive_with_display() {
        let config = ThinkingConfig::Adaptive {
            display: Some(ThinkingDisplay::Summarized),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""type":"adaptive""#));
        assert!(json.contains(r#""display":"summarized""#));
        let roundtrip: ThinkingConfig = serde_json::from_str(&json).unwrap();
        match roundtrip {
            ThinkingConfig::Adaptive { display } => {
                assert_eq!(display, Some(ThinkingDisplay::Summarized));
            }
            _ => panic!("Expected Adaptive variant"),
        }
    }
}
