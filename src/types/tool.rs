use serde::{Deserialize, Serialize};

use super::metadata::CacheControl;
use super::search::UserLocation;

/// A tool definition. Server tools (Bash, TextEditor, WebSearch) are distinguished
/// by their `type` field value. Custom tools are the catch-all for tools with an
/// `input_schema` field.
///
/// Serialization uses `#[serde(untagged)]`. Deserialization uses a custom
/// implementation that inspects the `type` field to dispatch to the correct variant.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum ToolDefinition {
    Bash(BashTool),
    TextEditor20250124(TextEditorTool),
    TextEditor20250429(TextEditorTool429),
    TextEditor20250728(TextEditorTool728),
    WebSearch(WebSearchTool),
    WebSearch20260209(WebSearchTool20260209),
    WebFetch20260209(WebFetchTool20260209),
    Custom(Tool),
}

impl<'de> Deserialize<'de> for ToolDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let type_field = value.get("type").and_then(|v| v.as_str());

        match type_field {
            Some("bash_20250124") => {
                let tool: BashTool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::Bash(tool))
            }
            Some("text_editor_20250124") => {
                let tool: TextEditorTool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::TextEditor20250124(tool))
            }
            Some("text_editor_20250429") => {
                let tool: TextEditorTool429 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::TextEditor20250429(tool))
            }
            Some("text_editor_20250728") => {
                let tool: TextEditorTool728 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::TextEditor20250728(tool))
            }
            Some("web_search_20250305") => {
                let tool: WebSearchTool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::WebSearch(tool))
            }
            Some("web_search_20260209") => {
                let tool: WebSearchTool20260209 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::WebSearch20260209(tool))
            }
            Some("web_fetch_20260209") => {
                let tool: WebFetchTool20260209 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::WebFetch20260209(tool))
            }
            _ => {
                // No type or unrecognized type -> Custom tool
                let tool: Tool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::Custom(tool))
            }
        }
    }
}

/// A custom tool definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: ToolInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eager_input_streaming: Option<bool>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,
}

/// The JSON Schema for a tool's input.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<serde_json::Value>,
}

/// A Bash server tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl BashTool {
    /// Create a new Bash tool with the standard type.
    pub fn new() -> Self {
        Self {
            tool_type: "bash_20250124".to_string(),
            name: "bash".to_string(),
            cache_control: None,
        }
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

/// A text editor server tool (2025-01-24 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl TextEditorTool {
    /// Create a new text editor tool (2025-01-24).
    pub fn new() -> Self {
        Self {
            tool_type: "text_editor_20250124".to_string(),
            name: "str_replace_editor".to_string(),
            cache_control: None,
        }
    }
}

impl Default for TextEditorTool {
    fn default() -> Self {
        Self::new()
    }
}

/// A text editor server tool (2025-04-29 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool429 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl TextEditorTool429 {
    /// Create a new text editor tool (2025-04-29).
    pub fn new() -> Self {
        Self {
            tool_type: "text_editor_20250429".to_string(),
            name: "str_replace_editor".to_string(),
            cache_control: None,
        }
    }
}

impl Default for TextEditorTool429 {
    fn default() -> Self {
        Self::new()
    }
}

/// A text editor server tool (2025-07-28 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool728 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl TextEditorTool728 {
    /// Create a new text editor tool (2025-07-28).
    pub fn new() -> Self {
        Self {
            tool_type: "text_editor_20250728".to_string(),
            name: "str_replace_editor".to_string(),
            cache_control: None,
        }
    }
}

impl Default for TextEditorTool728 {
    fn default() -> Self {
        Self::new()
    }
}

/// A web search server tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<UserLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl WebSearchTool {
    /// Create a new web search tool with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "web_search_20250305".to_string(),
            name: "WebSearch".to_string(),
            max_uses: None,
            allowed_domains: None,
            blocked_domains: None,
            user_location: None,
            cache_control: None,
            strict: None,
        }
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// A web search server tool (2026-02-09 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool20260209 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<UserLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
}

impl WebSearchTool20260209 {
    /// Create a new web search tool (2026-02-09) with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "web_search_20260209".to_string(),
            name: "web_search".to_string(),
            max_uses: None,
            allowed_domains: None,
            blocked_domains: None,
            user_location: None,
            cache_control: None,
            strict: None,
            allowed_callers: None,
            defer_loading: None,
        }
    }
}

impl Default for WebSearchTool20260209 {
    fn default() -> Self {
        Self::new()
    }
}

/// A web fetch server tool (2026-02-09 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTool20260209 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_content_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl WebFetchTool20260209 {
    /// Create a new web fetch tool (2026-02-09) with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "web_fetch_20260209".to_string(),
            name: "web_fetch".to_string(),
            max_content_tokens: None,
            max_uses: None,
            defer_loading: None,
            strict: None,
            allowed_domains: None,
            blocked_domains: None,
            allowed_callers: None,
            cache_control: None,
        }
    }
}

impl Default for WebFetchTool20260209 {
    fn default() -> Self {
        Self::new()
    }
}

/// How the model should choose which tool to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    Auto,
    Any,
    None,
    Tool { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_choice_auto() {
        let choice = ToolChoice::Auto;
        let json = serde_json::to_string(&choice).unwrap();
        assert_eq!(json, r#"{"type":"auto"}"#);
        let roundtrip: ToolChoice = serde_json::from_str(&json).unwrap();
        match roundtrip {
            ToolChoice::Auto => {}
            _ => panic!("Expected Auto variant"),
        }
    }

    #[test]
    fn test_tool_choice_any() {
        let choice = ToolChoice::Any;
        let json = serde_json::to_string(&choice).unwrap();
        assert_eq!(json, r#"{"type":"any"}"#);
    }

    #[test]
    fn test_tool_choice_none() {
        let choice = ToolChoice::None;
        let json = serde_json::to_string(&choice).unwrap();
        assert_eq!(json, r#"{"type":"none"}"#);
    }

    #[test]
    fn test_tool_choice_tool() {
        let choice = ToolChoice::Tool {
            name: "get_weather".to_string(),
        };
        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains(r#""type":"tool""#));
        assert!(json.contains(r#""name":"get_weather""#));
        let roundtrip: ToolChoice = serde_json::from_str(&json).unwrap();
        match roundtrip {
            ToolChoice::Tool { name } => assert_eq!(name, "get_weather"),
            _ => panic!("Expected Tool variant"),
        }
    }

    #[test]
    fn test_custom_tool_serialize() {
        let tool = ToolDefinition::Custom(Tool {
            name: "get_weather".to_string(),
            description: Some("Get current weather".to_string()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({
                    "location": {"type": "string"}
                })),
                required: Some(vec!["location".to_string()]),
                ..Default::default()
            },
            ..Default::default()
        });
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""name":"get_weather""#));
        assert!(json.contains(r#""description":"Get current weather""#));
        assert!(json.contains(r#""type":"object""#));
        assert!(!json.contains(r#""cache_control""#));
        assert!(!json.contains(r#""strict""#));
    }

    #[test]
    fn test_custom_tool_deserialize() {
        let json = r#"{
            "name": "calculator",
            "description": "A calculator",
            "input_schema": {
                "type": "object",
                "properties": {"expression": {"type": "string"}},
                "required": ["expression"]
            }
        }"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        match tool {
            ToolDefinition::Custom(t) => {
                assert_eq!(t.name, "calculator");
                assert_eq!(t.description.as_deref(), Some("A calculator"));
            }
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_bash_tool_serialize() {
        let tool = ToolDefinition::Bash(BashTool::new());
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"bash_20250124""#));
        assert!(json.contains(r#""name":"bash""#));
    }

    #[test]
    fn test_bash_tool_deserialize() {
        let json = r#"{"type":"bash_20250124","name":"bash"}"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        match tool {
            ToolDefinition::Bash(b) => {
                assert_eq!(b.tool_type, "bash_20250124");
                assert_eq!(b.name, "bash");
            }
            _ => panic!("Expected Bash variant"),
        }
    }

    #[test]
    fn test_text_editor_tool_serialize() {
        let tool = ToolDefinition::TextEditor20250124(TextEditorTool::new());
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"text_editor_20250124""#));
        assert!(json.contains(r#""name":"str_replace_editor""#));
    }

    #[test]
    fn test_text_editor_tool_deserialize() {
        let json = r#"{"type":"text_editor_20250124","name":"str_replace_editor"}"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        match tool {
            ToolDefinition::TextEditor20250124(t) => {
                assert_eq!(t.tool_type, "text_editor_20250124");
            }
            _ => panic!("Expected TextEditor20250124 variant"),
        }
    }

    #[test]
    fn test_text_editor_429_deserialize() {
        let json = r#"{"type":"text_editor_20250429","name":"str_replace_editor"}"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        match tool {
            ToolDefinition::TextEditor20250429(t) => {
                assert_eq!(t.tool_type, "text_editor_20250429");
            }
            _ => panic!("Expected TextEditor20250429 variant"),
        }
    }

    #[test]
    fn test_text_editor_728_deserialize() {
        let json = r#"{"type":"text_editor_20250728","name":"str_replace_editor"}"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        match tool {
            ToolDefinition::TextEditor20250728(t) => {
                assert_eq!(t.tool_type, "text_editor_20250728");
            }
            _ => panic!("Expected TextEditor20250728 variant"),
        }
    }

    #[test]
    fn test_web_search_tool_serialize() {
        let tool = ToolDefinition::WebSearch(WebSearchTool {
            max_uses: Some(5),
            allowed_domains: Some(vec!["example.com".to_string()]),
            ..WebSearchTool::new()
        });
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"web_search_20250305""#));
        assert!(json.contains(r#""name":"WebSearch""#));
        assert!(json.contains(r#""max_uses":5"#));
        assert!(json.contains(r#""allowed_domains":["example.com"]"#));
    }

    #[test]
    fn test_web_search_tool_deserialize() {
        let json = r#"{"type":"web_search_20250305","name":"web_search","max_uses":3}"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        match tool {
            ToolDefinition::WebSearch(w) => {
                assert_eq!(w.tool_type, "web_search_20250305");
                assert_eq!(w.max_uses, Some(3));
            }
            _ => panic!("Expected WebSearch variant"),
        }
    }

    #[test]
    fn test_tool_definition_dispatch_by_type() {
        // Server tools should deserialize to their specific variants
        let bash_json = r#"{"type":"bash_20250124","name":"bash"}"#;
        let tool: ToolDefinition = serde_json::from_str(bash_json).unwrap();
        assert!(matches!(tool, ToolDefinition::Bash(_)));

        let editor_json = r#"{"type":"text_editor_20250124","name":"str_replace_editor"}"#;
        let tool: ToolDefinition = serde_json::from_str(editor_json).unwrap();
        assert!(matches!(tool, ToolDefinition::TextEditor20250124(_)));

        let web_json = r#"{"type":"web_search_20250305","name":"web_search"}"#;
        let tool: ToolDefinition = serde_json::from_str(web_json).unwrap();
        assert!(matches!(tool, ToolDefinition::WebSearch(_)));

        // Custom tool (no type or "custom" type)
        let custom_json = r#"{"name":"calc","input_schema":{"type":"object"}}"#;
        let tool: ToolDefinition = serde_json::from_str(custom_json).unwrap();
        assert!(matches!(tool, ToolDefinition::Custom(_)));

        // Custom tool with explicit "custom" type
        let custom_typed = r#"{"type":"custom","name":"calc","input_schema":{"type":"object"}}"#;
        let tool: ToolDefinition = serde_json::from_str(custom_typed).unwrap();
        assert!(matches!(tool, ToolDefinition::Custom(_)));
    }

    #[test]
    fn test_tool_definition_roundtrip_all_variants() {
        let tools: Vec<ToolDefinition> = vec![
            ToolDefinition::Bash(BashTool::new()),
            ToolDefinition::TextEditor20250124(TextEditorTool::new()),
            ToolDefinition::TextEditor20250429(TextEditorTool429::new()),
            ToolDefinition::TextEditor20250728(TextEditorTool728::new()),
            ToolDefinition::WebSearch(WebSearchTool::new()),
            ToolDefinition::Custom(Tool {
                name: "test".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ];
        for tool in &tools {
            let json = serde_json::to_string(tool).unwrap();
            let _roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_tool_input_schema() {
        let schema = ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(serde_json::json!({
                "name": {"type": "string"},
                "age": {"type": "integer"}
            })),
            required: Some(vec!["name".to_string()]),
            ..Default::default()
        };
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains(r#""type":"object""#));
        assert!(json.contains(r#""required":["name"]"#));
    }

    #[test]
    fn test_tool_choice_roundtrip_all() {
        let choices = vec![
            ToolChoice::Auto,
            ToolChoice::Any,
            ToolChoice::None,
            ToolChoice::Tool {
                name: "test".to_string(),
            },
        ];
        for choice in choices {
            let json = serde_json::to_string(&choice).unwrap();
            let _roundtrip: ToolChoice = serde_json::from_str(&json).unwrap();
        }
    }
}
