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
    Bash20241022(BashTool20241022),
    TextEditor20241022(TextEditorTool20241022),
    TextEditor20250124(TextEditorTool),
    TextEditor20250429(TextEditorTool429),
    TextEditor20250728(TextEditorTool728),
    WebSearch(WebSearchTool),
    WebSearch20260209(WebSearchTool20260209),
    WebFetch20250910(WebFetchTool20250910),
    WebFetch20260209(WebFetchTool20260209),
    WebFetch20260309(WebFetchTool20260309),
    CodeExecution(CodeExecutionTool),
    CodeExecution20250522(CodeExecutionTool20250522),
    CodeExecution20260120(CodeExecutionTool20260120),
    ComputerUse20241022(ComputerTool20241022),
    ComputerUse20250124(ComputerTool20250124),
    ComputerUse20251124(ComputerTool20251124),
    Memory(MemoryTool),
    ToolSearchBm25(ToolSearchBm25Tool),
    ToolSearchRegex(ToolSearchRegexTool),
    McpToolset(McpToolset),
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
            Some("bash_20241022") => {
                let tool: BashTool20241022 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::Bash20241022(tool))
            }
            Some("bash_20250124") => {
                let tool: BashTool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::Bash(tool))
            }
            Some("text_editor_20241022") => {
                let tool: TextEditorTool20241022 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::TextEditor20241022(tool))
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
            Some("web_fetch_20250910") => {
                let tool: WebFetchTool20250910 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::WebFetch20250910(tool))
            }
            Some("web_fetch_20260209") => {
                let tool: WebFetchTool20260209 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::WebFetch20260209(tool))
            }
            Some("web_fetch_20260309") => {
                let tool: WebFetchTool20260309 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::WebFetch20260309(tool))
            }
            Some("code_execution_20250522") => {
                let tool: CodeExecutionTool20250522 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::CodeExecution20250522(tool))
            }
            Some("code_execution_20250825") => {
                let tool: CodeExecutionTool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::CodeExecution(tool))
            }
            Some("code_execution_20260120") => {
                let tool: CodeExecutionTool20260120 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::CodeExecution20260120(tool))
            }
            Some("computer_20241022") => {
                let tool: ComputerTool20241022 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::ComputerUse20241022(tool))
            }
            Some("computer_20250124") => {
                let tool: ComputerTool20250124 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::ComputerUse20250124(tool))
            }
            Some("computer_20251124") => {
                let tool: ComputerTool20251124 =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::ComputerUse20251124(tool))
            }
            Some("memory_20250818") => {
                let tool: MemoryTool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::Memory(tool))
            }
            Some("tool_search_bm25_20251119") => {
                let tool: ToolSearchBm25Tool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::ToolSearchBm25(tool))
            }
            Some("tool_search_regex_20251119") => {
                let tool: ToolSearchRegexTool =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::ToolSearchRegex(tool))
            }
            Some("mcp") => {
                let tool: McpToolset =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(ToolDefinition::McpToolset(tool))
            }
            _ => {
                // No type or unrecognized type -> Custom tool
                let tool: Tool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
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
            name: "web_search".to_string(),
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

/// A web fetch server tool (2025-09-10 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTool20250910 {
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
    pub citations: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl WebFetchTool20250910 {
    /// Create a new web fetch tool (2025-09-10) with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "web_fetch_20250910".to_string(),
            name: "web_fetch".to_string(),
            max_content_tokens: None,
            max_uses: None,
            defer_loading: None,
            strict: None,
            allowed_domains: None,
            blocked_domains: None,
            allowed_callers: None,
            citations: None,
            cache_control: None,
        }
    }
}

impl Default for WebFetchTool20250910 {
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
    pub citations: Option<serde_json::Value>,
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
            citations: None,
            cache_control: None,
        }
    }
}

impl Default for WebFetchTool20260209 {
    fn default() -> Self {
        Self::new()
    }
}

/// A code execution server tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl CodeExecutionTool {
    /// Create a new code execution tool with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "code_execution_20250825".to_string(),
            name: "code_execution".to_string(),
            max_uses: None,
            cache_control: None,
        }
    }
}

impl Default for CodeExecutionTool {
    fn default() -> Self {
        Self::new()
    }
}

/// A Bash server tool (2024-10-22 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashTool20241022 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl BashTool20241022 {
    /// Create a new Bash tool (2024-10-22 version).
    pub fn new() -> Self {
        Self {
            tool_type: "bash_20241022".to_string(),
            name: "bash".to_string(),
            cache_control: None,
        }
    }
}

impl Default for BashTool20241022 {
    fn default() -> Self {
        Self::new()
    }
}

/// A text editor server tool (2024-10-22 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool20241022 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl TextEditorTool20241022 {
    /// Create a new text editor tool (2024-10-22).
    pub fn new() -> Self {
        Self {
            tool_type: "text_editor_20241022".to_string(),
            name: "str_replace_editor".to_string(),
            cache_control: None,
        }
    }
}

impl Default for TextEditorTool20241022 {
    fn default() -> Self {
        Self::new()
    }
}

/// A web fetch server tool (2026-03-09 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTool20260309 {
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
    pub citations: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl WebFetchTool20260309 {
    /// Create a new web fetch tool (2026-03-09) with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "web_fetch_20260309".to_string(),
            name: "web_fetch".to_string(),
            max_content_tokens: None,
            max_uses: None,
            defer_loading: None,
            strict: None,
            allowed_domains: None,
            blocked_domains: None,
            allowed_callers: None,
            citations: None,
            cache_control: None,
        }
    }
}

impl Default for WebFetchTool20260309 {
    fn default() -> Self {
        Self::new()
    }
}

/// A code execution server tool (2025-05-22 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionTool20250522 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl CodeExecutionTool20250522 {
    /// Create a new code execution tool (2025-05-22) with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "code_execution_20250522".to_string(),
            name: "code_execution".to_string(),
            max_uses: None,
            cache_control: None,
        }
    }
}

impl Default for CodeExecutionTool20250522 {
    fn default() -> Self {
        Self::new()
    }
}

/// A code execution server tool (2026-01-20 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionTool20260120 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl CodeExecutionTool20260120 {
    /// Create a new code execution tool (2026-01-20) with defaults.
    pub fn new() -> Self {
        Self {
            tool_type: "code_execution_20260120".to_string(),
            name: "code_execution".to_string(),
            max_uses: None,
            cache_control: None,
        }
    }
}

impl Default for CodeExecutionTool20260120 {
    fn default() -> Self {
        Self::new()
    }
}

/// A computer use tool (2024-10-22 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerTool20241022 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    pub display_width_px: u32,
    pub display_height_px: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ComputerTool20241022 {
    /// Create a new computer use tool (2024-10-22).
    pub fn new(display_width_px: u32, display_height_px: u32) -> Self {
        Self {
            tool_type: "computer_20241022".to_string(),
            name: "computer".to_string(),
            display_width_px,
            display_height_px,
            display_number: None,
            cache_control: None,
        }
    }
}

/// A computer use tool (2025-01-24 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerTool20250124 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    pub display_width_px: u32,
    pub display_height_px: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ComputerTool20250124 {
    /// Create a new computer use tool (2025-01-24).
    pub fn new(display_width_px: u32, display_height_px: u32) -> Self {
        Self {
            tool_type: "computer_20250124".to_string(),
            name: "computer".to_string(),
            display_width_px,
            display_height_px,
            display_number: None,
            cache_control: None,
        }
    }
}

/// A computer use tool (2025-11-24 version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerTool20251124 {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    pub display_width_px: u32,
    pub display_height_px: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ComputerTool20251124 {
    /// Create a new computer use tool (2025-11-24).
    pub fn new(display_width_px: u32, display_height_px: u32) -> Self {
        Self {
            tool_type: "computer_20251124".to_string(),
            name: "computer".to_string(),
            display_width_px,
            display_height_px,
            display_number: None,
            cache_control: None,
        }
    }
}

/// A memory server tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl MemoryTool {
    /// Create a new memory tool.
    pub fn new() -> Self {
        Self {
            tool_type: "memory_20250818".to_string(),
            name: "memory".to_string(),
            cache_control: None,
        }
    }
}

impl Default for MemoryTool {
    fn default() -> Self {
        Self::new()
    }
}

/// A tool search tool definition (BM25 variant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchBm25Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ToolSearchBm25Tool {
    /// Create a new BM25 tool search tool definition.
    pub fn new() -> Self {
        Self {
            tool_type: "tool_search_bm25_20251119".to_string(),
            name: "tool_search".to_string(),
            max_results: None,
            cache_control: None,
        }
    }
}

impl Default for ToolSearchBm25Tool {
    fn default() -> Self {
        Self::new()
    }
}

/// A tool search tool definition (Regex variant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchRegexTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ToolSearchRegexTool {
    /// Create a new Regex tool search tool definition.
    pub fn new() -> Self {
        Self {
            tool_type: "tool_search_regex_20251119".to_string(),
            name: "tool_search".to_string(),
            max_results: None,
            cache_control: None,
        }
    }
}

impl Default for ToolSearchRegexTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool configuration for MCP toolsets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// An MCP (Model Context Protocol) toolset definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolset {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub server_label: String,
    pub server_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_configuration: Option<McpToolConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl McpToolset {
    /// Create a new MCP toolset.
    pub fn new(server_label: impl Into<String>, server_url: impl Into<String>) -> Self {
        Self {
            tool_type: "mcp".to_string(),
            server_label: server_label.into(),
            server_url: server_url.into(),
            allowed_tools: None,
            tool_configuration: None,
            cache_control: None,
        }
    }
}

/// How the model should choose which tool to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    Auto {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    Any {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    None,
    Tool {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_choice_auto() {
        let choice = ToolChoice::Auto {
            disable_parallel_tool_use: None,
        };
        let json = serde_json::to_string(&choice).unwrap();
        assert_eq!(json, r#"{"type":"auto"}"#);
        let roundtrip: ToolChoice = serde_json::from_str(&json).unwrap();
        match roundtrip {
            ToolChoice::Auto { .. } => {}
            _ => panic!("Expected Auto variant"),
        }
    }

    #[test]
    fn test_tool_choice_auto_disable_parallel() {
        let choice = ToolChoice::Auto {
            disable_parallel_tool_use: Some(true),
        };
        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains(r#""type":"auto""#));
        assert!(json.contains(r#""disable_parallel_tool_use":true"#));
    }

    #[test]
    fn test_tool_choice_any() {
        let choice = ToolChoice::Any {
            disable_parallel_tool_use: None,
        };
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
            disable_parallel_tool_use: None,
        };
        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains(r#""type":"tool""#));
        assert!(json.contains(r#""name":"get_weather""#));
        let roundtrip: ToolChoice = serde_json::from_str(&json).unwrap();
        match roundtrip {
            ToolChoice::Tool { name, .. } => assert_eq!(name, "get_weather"),
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
        assert!(json.contains(r#""name":"web_search""#));
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
            ToolDefinition::Bash20241022(BashTool20241022::new()),
            ToolDefinition::TextEditor20241022(TextEditorTool20241022::new()),
            ToolDefinition::TextEditor20250124(TextEditorTool::new()),
            ToolDefinition::TextEditor20250429(TextEditorTool429::new()),
            ToolDefinition::TextEditor20250728(TextEditorTool728::new()),
            ToolDefinition::WebSearch(WebSearchTool::new()),
            ToolDefinition::WebFetch20260309(WebFetchTool20260309::new()),
            ToolDefinition::CodeExecution(CodeExecutionTool::new()),
            ToolDefinition::CodeExecution20250522(CodeExecutionTool20250522::new()),
            ToolDefinition::CodeExecution20260120(CodeExecutionTool20260120::new()),
            ToolDefinition::ComputerUse20241022(ComputerTool20241022::new(1920, 1080)),
            ToolDefinition::ComputerUse20250124(ComputerTool20250124::new(1920, 1080)),
            ToolDefinition::ComputerUse20251124(ComputerTool20251124::new(1920, 1080)),
            ToolDefinition::Memory(MemoryTool::new()),
            ToolDefinition::ToolSearchBm25(ToolSearchBm25Tool::new()),
            ToolDefinition::ToolSearchRegex(ToolSearchRegexTool::new()),
            ToolDefinition::McpToolset(McpToolset::new("test-server", "https://mcp.example.com")),
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
            ToolChoice::Auto {
                disable_parallel_tool_use: None,
            },
            ToolChoice::Any {
                disable_parallel_tool_use: None,
            },
            ToolChoice::None,
            ToolChoice::Tool {
                name: "test".to_string(),
                disable_parallel_tool_use: None,
            },
        ];
        for choice in choices {
            let json = serde_json::to_string(&choice).unwrap();
            let _roundtrip: ToolChoice = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_computer_tool_serialize() {
        let tool = ToolDefinition::ComputerUse20241022(ComputerTool20241022::new(1920, 1080));
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"computer_20241022""#));
        assert!(json.contains(r#""display_width_px":1920"#));
        assert!(json.contains(r#""display_height_px":1080"#));
    }

    #[test]
    fn test_computer_tool_deserialize() {
        let json = r#"{"type":"computer_20241022","name":"computer","display_width_px":1920,"display_height_px":1080}"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        assert!(matches!(tool, ToolDefinition::ComputerUse20241022(_)));
    }

    #[test]
    fn test_memory_tool_roundtrip() {
        let tool = ToolDefinition::Memory(MemoryTool::new());
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"memory_20250818""#));
        assert!(json.contains(r#""name":"memory""#));
        let roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert!(matches!(roundtrip, ToolDefinition::Memory(_)));
    }

    #[test]
    fn test_tool_search_bm25_roundtrip() {
        let tool = ToolDefinition::ToolSearchBm25(ToolSearchBm25Tool {
            max_results: Some(10),
            ..ToolSearchBm25Tool::new()
        });
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"tool_search_bm25_20251119""#));
        assert!(json.contains(r#""max_results":10"#));
        let roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert!(matches!(roundtrip, ToolDefinition::ToolSearchBm25(_)));
    }

    #[test]
    fn test_mcp_toolset_roundtrip() {
        let tool =
            ToolDefinition::McpToolset(McpToolset::new("my-server", "https://mcp.example.com/sse"));
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"mcp""#));
        assert!(json.contains(r#""server_label":"my-server""#));
        assert!(json.contains(r#""server_url":"https://mcp.example.com/sse""#));
        let roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert!(matches!(roundtrip, ToolDefinition::McpToolset(_)));
    }

    #[test]
    fn test_code_execution_20250522_roundtrip() {
        let tool = ToolDefinition::CodeExecution20250522(CodeExecutionTool20250522::new());
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"code_execution_20250522""#));
        let roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            roundtrip,
            ToolDefinition::CodeExecution20250522(_)
        ));
    }

    #[test]
    fn test_code_execution_20260120_roundtrip() {
        let tool = ToolDefinition::CodeExecution20260120(CodeExecutionTool20260120::new());
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"code_execution_20260120""#));
        let roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            roundtrip,
            ToolDefinition::CodeExecution20260120(_)
        ));
    }

    #[test]
    fn test_bash_20241022_roundtrip() {
        let tool = ToolDefinition::Bash20241022(BashTool20241022::new());
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"bash_20241022""#));
        let roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert!(matches!(roundtrip, ToolDefinition::Bash20241022(_)));
    }

    #[test]
    fn test_text_editor_20241022_roundtrip() {
        let tool = ToolDefinition::TextEditor20241022(TextEditorTool20241022::new());
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"text_editor_20241022""#));
        let roundtrip: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert!(matches!(roundtrip, ToolDefinition::TextEditor20241022(_)));
    }
}
