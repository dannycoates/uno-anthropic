use serde::{Deserialize, Serialize};

use super::document::DocumentSource;
use super::metadata::CacheControl;

// ── Response content blocks ──────────────────────────────────────────

/// A content block in a message response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text(TextBlock),
    Thinking(ThinkingBlock),
    RedactedThinking(RedactedThinkingBlock),
    ToolUse(ToolUseBlock),
    ServerToolUse(ServerToolUseBlock),
    WebSearchToolResult(WebSearchToolResultBlock),
    ContainerUpload(ContainerUploadBlock),
    WebFetchToolResult(WebFetchToolResultBlock),
    ToolSearchToolResult(ToolSearchToolResultBlock),
}

/// A text content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<super::citation::TextCitation>>,
}

/// A thinking content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    pub thinking: String,
    pub signature: String,
}

/// A redacted thinking content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedThinkingBlock {
    pub data: String,
}

/// A tool use content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    /// Accumulated partial JSON during streaming. Not part of the API response.
    #[serde(skip)]
    pub partial_json: Option<String>,
}

/// A server tool use content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// A web search tool result content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultBlock {
    pub tool_use_id: String,
    pub content: WebSearchToolResultContent,
}

/// Content of a web search tool result: either search results or an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebSearchToolResultContent {
    Results(Vec<WebSearchResultBlock>),
    Error(WebSearchToolRequestError),
}

/// A single web search result block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResultBlock {
    #[serde(rename = "type")]
    pub result_type: String,
    pub url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_age: Option<String>,
}

/// An error from a web search tool request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolRequestError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error_code: WebSearchToolResultErrorCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Typed error codes for web search tool results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    MaxUsesExceeded,
    TooManyRequests,
    QueryTooLong,
    RequestTooLarge,
}

/// Typed error codes for web fetch tool results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebFetchToolResultErrorCode {
    InvalidToolInput,
    UrlTooLong,
    UrlNotAllowed,
    UrlNotAccessible,
    UnsupportedContentType,
    TooManyRequests,
    MaxUsesExceeded,
    Unavailable,
}

/// A container upload content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerUploadBlock {
    pub file_id: String,
}

/// A web fetch tool result content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchToolResultBlock {
    pub tool_use_id: String,
    pub content: WebFetchToolResultContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<serde_json::Value>,
}

/// Content of a web fetch tool result: either a fetched page or an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebFetchToolResultContent {
    Success(WebFetchBlock),
    Error(WebFetchToolResultErrorBlock),
}

/// A successful web fetch result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchBlock {
    pub url: String,
    pub content: DocumentSource,
    pub retrieved_at: String,
}

/// An error from a web fetch tool request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchToolResultErrorBlock {
    pub error_code: WebFetchToolResultErrorCode,
}

/// A tool search tool result content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolResultBlock {
    pub tool_use_id: String,
    pub content: ToolSearchToolResultContent,
}

/// Content of a tool search tool result: either search results or an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolSearchToolResultContent {
    SearchResult(ToolSearchToolSearchResultBlock),
    Error(ToolSearchToolResultError),
}

/// A successful tool search result block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolSearchResultBlock {
    pub tool_references: Vec<ToolReferenceBlock>,
}

/// A tool reference within a tool search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReferenceBlock {
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// An error from a tool search tool request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolResultError {
    pub error_code: ToolSearchToolResultErrorCode,
    pub error_message: String,
}

/// Typed error codes for tool search tool results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSearchToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
}

// ── Request content blocks ───────────────────────────────────────────

/// A content block in a message request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockParam {
    Text(TextBlockParam),
    Image(ImageBlockParam),
    Document(DocumentBlockParam),
    ToolUse(ToolUseBlockParam),
    ToolResult(ToolResultBlockParam),
    Thinking(ThinkingBlockParam),
    RedactedThinking(RedactedThinkingBlockParam),
    ServerToolUse(ServerToolUseBlockParam),
    WebSearchToolResult(WebSearchToolResultBlockParam),
    SearchResult(SearchResultBlockParam),
    ContainerUpload(ContainerUploadBlockParam),
    WebFetchToolResult(WebFetchToolResultBlockParam),
    ToolSearchToolResult(ToolSearchToolResultBlockParam),
}

/// A text block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlockParam {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<super::citation::CitationsConfig>,
}

impl TextBlockParam {
    /// Create a new text block with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            cache_control: None,
            citations: None,
        }
    }
}

/// An image block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlockParam {
    pub source: super::image::ImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A document block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBlockParam {
    pub source: super::document::DocumentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<super::citation::CitationsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A tool use block in a request (for multi-turn conversations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlockParam {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A tool result block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlockParam {
    pub tool_use_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Content of a tool result: either a plain string or content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ToolResultContentBlock>),
}

/// A content block allowed inside a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContentBlock {
    Text(TextBlockParam),
    Image(ImageBlockParam),
    Document(DocumentBlockParam),
    SearchResult(SearchResultBlockParam),
}

/// A thinking block in a request (for multi-turn conversations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlockParam {
    pub thinking: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A redacted thinking block in a request (for multi-turn conversations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedThinkingBlockParam {
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A server tool use block in a request (for multi-turn conversations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUseBlockParam {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A web search tool result block in a request (for multi-turn conversations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultBlockParam {
    pub tool_use_id: String,
    pub content: WebSearchToolResultContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A search result block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultBlockParam {
    pub source: String,
    pub title: String,
    pub content: Vec<SearchResultTextContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<super::citation::CitationsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Text content within a search result block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultTextContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// A container upload block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerUploadBlockParam {
    pub file_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A web fetch tool result block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchToolResultBlockParam {
    pub tool_use_id: String,
    pub content: WebFetchToolResultContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// A tool search tool result block in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolResultBlockParam {
    pub tool_use_id: String,
    pub content: ToolSearchToolResultContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl From<&str> for ToolResultContent {
    fn from(s: &str) -> Self {
        ToolResultContent::Text(s.to_string())
    }
}

impl From<String> for ToolResultContent {
    fn from(s: String) -> Self {
        ToolResultContent::Text(s)
    }
}

impl From<Vec<ToolResultContentBlock>> for ToolResultContent {
    fn from(blocks: Vec<ToolResultContentBlock>) -> Self {
        ToolResultContent::Blocks(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_block_text() {
        let json = r#"{"type":"text","text":"Hello, world!"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match &block {
            ContentBlock::Text(t) => assert_eq!(t.text, "Hello, world!"),
            _ => panic!("Expected Text variant"),
        }
        let roundtrip = serde_json::to_string(&block).unwrap();
        let block2: ContentBlock = serde_json::from_str(&roundtrip).unwrap();
        match block2 {
            ContentBlock::Text(t) => assert_eq!(t.text, "Hello, world!"),
            _ => panic!("Expected Text variant after roundtrip"),
        }
    }

    #[test]
    fn test_content_block_thinking() {
        let json = r#"{"type":"thinking","thinking":"Let me think...","signature":"sig123"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match &block {
            ContentBlock::Thinking(t) => {
                assert_eq!(t.thinking, "Let me think...");
                assert_eq!(t.signature, "sig123");
            }
            _ => panic!("Expected Thinking variant"),
        }
    }

    #[test]
    fn test_content_block_redacted_thinking() {
        let json = r#"{"type":"redacted_thinking","data":"abc123"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::RedactedThinking(r) => assert_eq!(r.data, "abc123"),
            _ => panic!("Expected RedactedThinking variant"),
        }
    }

    #[test]
    fn test_content_block_tool_use() {
        let json = r#"{"type":"tool_use","id":"tu_123","name":"get_weather","input":{"location":"SF"}}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::ToolUse(t) => {
                assert_eq!(t.id, "tu_123");
                assert_eq!(t.name, "get_weather");
                assert_eq!(t.input["location"], "SF");
            }
            _ => panic!("Expected ToolUse variant"),
        }
    }

    #[test]
    fn test_content_block_server_tool_use() {
        let json = r#"{"type":"server_tool_use","id":"stu_123","name":"web_search","input":{"query":"rust lang"}}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::ServerToolUse(s) => {
                assert_eq!(s.id, "stu_123");
                assert_eq!(s.name, "web_search");
            }
            _ => panic!("Expected ServerToolUse variant"),
        }
    }

    #[test]
    fn test_content_block_param_text() {
        let param = ContentBlockParam::Text(TextBlockParam::new("Hello"));
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains(r#""text":"Hello""#));
        let roundtrip: ContentBlockParam = serde_json::from_str(&json).unwrap();
        match roundtrip {
            ContentBlockParam::Text(t) => assert_eq!(t.text, "Hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_content_block_param_tool_result_text() {
        let param = ContentBlockParam::ToolResult(ToolResultBlockParam {
            tool_use_id: "tu_123".to_string(),
            content: Some(ToolResultContent::Text("result text".to_string())),
            is_error: None,
            cache_control: None,
        });
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains(r#""type":"tool_result""#));
        assert!(json.contains(r#""tool_use_id":"tu_123""#));
        assert!(json.contains(r#""content":"result text""#));
    }

    #[test]
    fn test_tool_result_content_text() {
        let content = ToolResultContent::Text("hello".to_string());
        let json = serde_json::to_string(&content).unwrap();
        assert_eq!(json, r#""hello""#);
    }

    #[test]
    fn test_tool_result_content_blocks() {
        let blocks = ToolResultContent::Blocks(vec![ToolResultContentBlock::Text(
            TextBlockParam::new("result"),
        )]);
        let json = serde_json::to_string(&blocks).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains(r#""text":"result""#));
    }

    #[test]
    fn test_tool_result_content_from_str() {
        let content: ToolResultContent = "hello".into();
        match content {
            ToolResultContent::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_web_search_tool_result_content_results() {
        let json = r#"[{"type":"web_search_result","url":"https://example.com","title":"Example","encrypted_content":"enc123"}]"#;
        let content: WebSearchToolResultContent = serde_json::from_str(json).unwrap();
        match content {
            WebSearchToolResultContent::Results(results) => {
                assert_eq!(results.len(), 1);
                assert_eq!(results[0].url, "https://example.com");
                assert_eq!(results[0].title, "Example");
            }
            _ => panic!("Expected Results variant"),
        }
    }

    #[test]
    fn test_web_search_tool_result_content_error() {
        let json = r#"{"type":"web_search_error","error_code":"max_uses_exceeded"}"#;
        let content: WebSearchToolResultContent = serde_json::from_str(json).unwrap();
        match content {
            WebSearchToolResultContent::Error(err) => {
                assert!(matches!(
                    err.error_code,
                    WebSearchToolResultErrorCode::MaxUsesExceeded
                ));
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_text_block_param_new() {
        let block = TextBlockParam::new("Hello");
        assert_eq!(block.text, "Hello");
        assert!(block.cache_control.is_none());
        assert!(block.citations.is_none());
    }

    #[test]
    fn test_content_block_param_tool_use() {
        let json = r#"{"type":"tool_use","id":"tu_1","name":"calc","input":{"x":1}}"#;
        let param: ContentBlockParam = serde_json::from_str(json).unwrap();
        match param {
            ContentBlockParam::ToolUse(t) => {
                assert_eq!(t.id, "tu_1");
                assert_eq!(t.name, "calc");
            }
            _ => panic!("Expected ToolUse variant"),
        }
    }

    #[test]
    fn test_content_block_param_thinking() {
        let json = r#"{"type":"thinking","thinking":"hmm","signature":"sig"}"#;
        let param: ContentBlockParam = serde_json::from_str(json).unwrap();
        match param {
            ContentBlockParam::Thinking(t) => {
                assert_eq!(t.thinking, "hmm");
                assert_eq!(t.signature, "sig");
            }
            _ => panic!("Expected Thinking variant"),
        }
    }

    #[test]
    fn test_content_block_param_redacted_thinking() {
        let json = r#"{"type":"redacted_thinking","data":"redacted"}"#;
        let param: ContentBlockParam = serde_json::from_str(json).unwrap();
        match param {
            ContentBlockParam::RedactedThinking(r) => assert_eq!(r.data, "redacted"),
            _ => panic!("Expected RedactedThinking variant"),
        }
    }
}
