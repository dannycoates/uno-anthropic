use serde::{Deserialize, Serialize};

/// Configuration for extended thinking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingConfig {
    /// Enable extended thinking with a token budget.
    Enabled { budget_tokens: u32 },
    /// Disable extended thinking.
    Disabled,
    /// Adaptive thinking (no budget_tokens field).
    Adaptive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_enabled() {
        let config = ThinkingConfig::Enabled {
            budget_tokens: 10000,
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
        let config = ThinkingConfig::Adaptive;
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"adaptive"}"#);
    }

    #[test]
    fn test_deserialize_enabled() {
        let json = r#"{"type":"enabled","budget_tokens":5000}"#;
        let config: ThinkingConfig = serde_json::from_str(json).unwrap();
        match config {
            ThinkingConfig::Enabled { budget_tokens } => assert_eq!(budget_tokens, 5000),
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
        assert!(matches!(config, ThinkingConfig::Adaptive));
    }

    #[test]
    fn test_roundtrip_all_variants() {
        let variants = vec![
            ThinkingConfig::Enabled {
                budget_tokens: 8192,
            },
            ThinkingConfig::Disabled,
            ThinkingConfig::Adaptive,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: ThinkingConfig = serde_json::from_str(&json).unwrap();
            // Compare serialized forms since ThinkingConfig doesn't derive PartialEq
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }
}
