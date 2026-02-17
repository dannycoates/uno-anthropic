use serde::Serialize;

use crate::types::message::{MessageParam, SystemContent};
use crate::types::metadata::{Metadata, OutputConfig, ServiceTier};
use crate::types::model::Model;
use crate::types::thinking::ThinkingConfig;
use crate::types::tool::{ToolChoice, ToolDefinition};

/// Parameters for creating a message.
///
/// Use the builder pattern via `MessageCreateParams::builder()`:
/// ```ignore
/// let params = MessageCreateParams::builder()
///     .model(Model::ClaudeOpus4_6)
///     .max_tokens(1024)
///     .messages(vec![MessageParam::user("Hello")])
///     .build();
/// ```
///
/// The `stream` field is not exposed; it is injected internally by
/// `create()` (false) and `create_stream()` (true).
#[derive(Debug, Clone, Serialize, bon::Builder)]
pub struct MessageCreateParams {
    pub model: Model,
    pub max_tokens: u32,
    pub messages: Vec<MessageParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,
}

/// Parameters for counting tokens.
///
/// Use the builder pattern via `CountTokensParams::builder()`:
/// ```ignore
/// let params = CountTokensParams::builder()
///     .model(Model::ClaudeOpus4_6)
///     .messages(vec![MessageParam::user("Hello")])
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, bon::Builder)]
pub struct CountTokensParams {
    pub model: Model,
    pub messages: Vec<MessageParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_create_params_minimal() {
        let params = MessageCreateParams::builder()
            .model(Model::ClaudeOpus4_6)
            .max_tokens(1024)
            .messages(vec![MessageParam::user("Hello")])
            .build();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""model":"claude-opus-4-6""#));
        assert!(json.contains(r#""max_tokens":1024"#));
        assert!(json.contains(r#""content":"Hello""#));
        // Optional fields should not appear
        assert!(!json.contains("temperature"));
        assert!(!json.contains("top_p"));
        assert!(!json.contains("top_k"));
        assert!(!json.contains("stop_sequences"));
        assert!(!json.contains("metadata"));
        assert!(!json.contains("thinking"));
        assert!(!json.contains("tool_choice"));
        assert!(!json.contains("tools"));
        assert!(!json.contains("stream"));
    }

    #[test]
    fn test_message_create_params_with_optionals() {
        let params = MessageCreateParams::builder()
            .model(Model::ClaudeSonnet4_5)
            .max_tokens(2048)
            .messages(vec![MessageParam::user("Test")])
            .system(SystemContent::from("You are helpful."))
            .temperature(0.7)
            .top_p(0.9)
            .top_k(40)
            .stop_sequences(vec!["STOP".to_string()])
            .thinking(ThinkingConfig::Enabled {
                budget_tokens: 5000,
            })
            .service_tier(ServiceTier::Auto)
            .build();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""system":"You are helpful.""#));
        assert!(json.contains(r#""temperature":0.7"#));
        assert!(json.contains(r#""top_p":0.9"#));
        assert!(json.contains(r#""top_k":40"#));
        assert!(json.contains(r#""stop_sequences":["STOP"]"#));
        assert!(json.contains(r#""budget_tokens":5000"#));
        assert!(json.contains(r#""service_tier":"auto""#));
    }

    #[test]
    fn test_message_create_params_no_stream_field() {
        let params = MessageCreateParams::builder()
            .model(Model::ClaudeOpus4_6)
            .max_tokens(100)
            .messages(vec![MessageParam::user("Hi")])
            .build();
        let json = serde_json::to_string(&params).unwrap();
        assert!(!json.contains("stream"));
    }

    #[test]
    fn test_count_tokens_params_minimal() {
        let params = CountTokensParams::builder()
            .model(Model::ClaudeOpus4_6)
            .messages(vec![MessageParam::user("Hello")])
            .build();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""model":"claude-opus-4-6""#));
        assert!(json.contains(r#""content":"Hello""#));
        assert!(!json.contains("system"));
        assert!(!json.contains("tools"));
        assert!(!json.contains("tool_choice"));
        assert!(!json.contains("thinking"));
    }

    #[test]
    fn test_count_tokens_params_with_system() {
        let params = CountTokensParams::builder()
            .model(Model::ClaudeSonnet4_5)
            .messages(vec![MessageParam::user("Hi")])
            .system(SystemContent::from("Be concise."))
            .build();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""system":"Be concise.""#));
    }

    #[test]
    fn test_count_tokens_params_with_tools() {
        use crate::types::tool::{Tool, ToolInputSchema};
        let params = CountTokensParams::builder()
            .model(Model::ClaudeOpus4_6)
            .messages(vec![MessageParam::user("What's the weather?")])
            .tools(vec![ToolDefinition::Custom(Tool {
                name: "get_weather".to_string(),
                description: Some("Get weather".to_string()),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: Some(serde_json::json!({"location": {"type": "string"}})),
                    required: Some(vec!["location".to_string()]),
                    ..Default::default()
                },
                ..Default::default()
            })])
            .build();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""name":"get_weather""#));
    }
}
