use serde::{Deserialize, Serialize};

use super::common::{Role, StopReason};
use super::content::{ContentBlock, ContentBlockParam, TextBlockParam};
use super::usage::Usage;

/// A message response from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: Role,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<StopReason>,
    #[serde(default)]
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

impl Message {
    /// Convert this response message into a `MessageParam` for multi-turn conversations.
    pub fn to_param(&self) -> MessageParam {
        MessageParam {
            role: self.role.clone(),
            content: MessageContent::Blocks(
                self.content
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text(t) => ContentBlockParam::Text(TextBlockParam {
                            text: t.text.clone(),
                            cache_control: None,
                            citations: None,
                        }),
                        ContentBlock::ToolUse(t) => {
                            ContentBlockParam::ToolUse(super::content::ToolUseBlockParam {
                                id: t.id.clone(),
                                name: t.name.clone(),
                                input: t.input.clone(),
                                cache_control: None,
                            })
                        }
                        ContentBlock::Thinking(t) => {
                            ContentBlockParam::Thinking(super::content::ThinkingBlockParam {
                                thinking: t.thinking.clone(),
                                signature: t.signature.clone(),
                                cache_control: None,
                            })
                        }
                        ContentBlock::RedactedThinking(r) => {
                            ContentBlockParam::RedactedThinking(
                                super::content::RedactedThinkingBlockParam {
                                    data: r.data.clone(),
                                    cache_control: None,
                                },
                            )
                        }
                        ContentBlock::ServerToolUse(s) => {
                            ContentBlockParam::ServerToolUse(
                                super::content::ServerToolUseBlockParam {
                                    id: s.id.clone(),
                                    name: s.name.clone(),
                                    input: s.input.clone(),
                                    cache_control: None,
                                },
                            )
                        }
                        ContentBlock::WebSearchToolResult(w) => {
                            ContentBlockParam::WebSearchToolResult(
                                super::content::WebSearchToolResultBlockParam {
                                    tool_use_id: w.tool_use_id.clone(),
                                    content: w.content.clone(),
                                    cache_control: None,
                                },
                            )
                        }
                        ContentBlock::ContainerUpload(c) => {
                            ContentBlockParam::ContainerUpload(
                                super::content::ContainerUploadBlockParam {
                                    file_id: c.file_id.clone(),
                                    cache_control: None,
                                },
                            )
                        }
                        ContentBlock::WebFetchToolResult(w) => {
                            ContentBlockParam::WebFetchToolResult(
                                super::content::WebFetchToolResultBlockParam {
                                    tool_use_id: w.tool_use_id.clone(),
                                    content: w.content.clone(),
                                    cache_control: None,
                                },
                            )
                        }
                        ContentBlock::ToolSearchToolResult(t) => {
                            ContentBlockParam::ToolSearchToolResult(
                                super::content::ToolSearchToolResultBlockParam {
                                    tool_use_id: t.tool_use_id.clone(),
                                    content: t.content.clone(),
                                    cache_control: None,
                                },
                            )
                        }
                    })
                    .collect(),
            ),
        }
    }
}

/// A message parameter for a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageParam {
    pub role: Role,
    pub content: MessageContent,
}

impl MessageParam {
    /// Create a user message from a string.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Text(text.into()),
        }
    }

    /// Create an assistant message from a string.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Text(text.into()),
        }
    }

    /// Create a user message from content blocks.
    pub fn user_blocks(blocks: Vec<ContentBlockParam>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Blocks(blocks),
        }
    }

    /// Create an assistant message from content blocks.
    pub fn assistant_blocks(blocks: Vec<ContentBlockParam>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Blocks(blocks),
        }
    }
}

/// Message content: either a plain string or structured content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlockParam>),
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        MessageContent::Text(s.to_string())
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        MessageContent::Text(s)
    }
}

impl From<Vec<ContentBlockParam>> for MessageContent {
    fn from(blocks: Vec<ContentBlockParam>) -> Self {
        MessageContent::Blocks(blocks)
    }
}

/// System content: either a plain string or text blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemContent {
    Text(String),
    Blocks(Vec<TextBlockParam>),
}

impl From<&str> for SystemContent {
    fn from(s: &str) -> Self {
        SystemContent::Text(s.to_string())
    }
}

impl From<String> for SystemContent {
    fn from(s: String) -> Self {
        SystemContent::Text(s)
    }
}

impl From<Vec<TextBlockParam>> for SystemContent {
    fn from(blocks: Vec<TextBlockParam>) -> Self {
        SystemContent::Blocks(blocks)
    }
}

/// Response from the count_tokens endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenCount {
    pub input_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_param_user() {
        let param = MessageParam::user("Hello");
        assert_eq!(param.role, Role::User);
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains(r#""role":"user""#));
        assert!(json.contains(r#""content":"Hello""#));
    }

    #[test]
    fn test_message_param_assistant() {
        let param = MessageParam::assistant("Hi there");
        assert_eq!(param.role, Role::Assistant);
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains(r#""role":"assistant""#));
    }

    #[test]
    fn test_message_content_text() {
        let content = MessageContent::Text("hello".to_string());
        let json = serde_json::to_string(&content).unwrap();
        assert_eq!(json, r#""hello""#);
    }

    #[test]
    fn test_message_content_blocks() {
        let content =
            MessageContent::Blocks(vec![ContentBlockParam::Text(TextBlockParam::new("hi"))]);
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"text""#));
    }

    #[test]
    fn test_system_content_from_str() {
        let content: SystemContent = "You are helpful".into();
        let json = serde_json::to_string(&content).unwrap();
        assert_eq!(json, r#""You are helpful""#);
    }

    #[test]
    fn test_system_content_from_blocks() {
        let blocks = vec![TextBlockParam::new("System prompt")];
        let content = SystemContent::from(blocks);
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""text":"System prompt""#));
    }

    #[test]
    fn test_deserialize_message() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-opus-4-6",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, "msg_123");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content.len(), 1);
        assert_eq!(msg.stop_reason, Some(StopReason::EndTurn));
        assert_eq!(msg.usage.input_tokens, 10);
    }

    #[test]
    fn test_message_to_param() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-opus-4-6",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        let param = msg.to_param();
        assert_eq!(param.role, Role::Assistant);
        let param_json = serde_json::to_string(&param).unwrap();
        assert!(param_json.contains(r#""text":"Hello!""#));
    }
}
