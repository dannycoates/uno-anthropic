use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use futures::StreamExt;
use pin_project_lite::pin_project;
use serde::Deserialize;

use crate::error::Error;
use crate::streaming::sse::{RawSseEvent, parse_sse_stream};
use crate::types::common::StopReason;
use crate::types::content::ContentBlock;
use crate::types::message::Message;
use crate::types::usage::MessageDeltaUsage;

/// SSE event deserialized from the stream. Dispatched by `event:` field name.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart {
        message: Message,
    },
    ContentBlockStart {
        index: u32,
        content_block: ContentBlock,
    },
    ContentBlockDelta {
        index: u32,
        delta: ContentBlockDelta,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        delta: MessageDelta,
        usage: MessageDeltaUsage,
    },
    MessageStop,
    Ping,
    Error {
        error: crate::error::ApiErrorBody,
    },
}

/// Delta types for streaming content blocks.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockDelta {
    TextDelta {
        text: String,
    },
    InputJsonDelta {
        partial_json: String,
    },
    ThinkingDelta {
        thinking: String,
    },
    SignatureDelta {
        signature: String,
    },
    CitationsDelta {
        citation: serde_json::Value, // TextCitation, kept as Value to avoid circular dep issues
    },
}

/// Delta information in a `message_delta` streaming event.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDelta {
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
}

/// Map an SSE event type string to the correct StreamEvent variant by parsing the data as JSON.
fn parse_stream_event(raw: RawSseEvent) -> Result<StreamEvent, Error> {
    let event_type = raw.event.as_deref().unwrap_or("");
    let data = raw.data.as_deref().unwrap_or("{}");

    // The SSE `event:` field tells us the type, and data is the JSON payload.
    // We need to inject the type field into the JSON for serde to dispatch correctly.
    let mut value: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| Error::StreamError(format!("Failed to parse SSE data as JSON: {e}")))?;

    if let Some(obj) = value.as_object_mut() {
        obj.insert("type".to_string(), serde_json::Value::String(event_type.to_string()));
    }

    let event: StreamEvent = serde_json::from_value(value)
        .map_err(|e| Error::StreamError(format!("Failed to deserialize stream event '{event_type}': {e}")))?;

    Ok(event)
}

pin_project! {
    /// A stream of `StreamEvent` items from a streaming Messages API response.
    ///
    /// Wraps an inner SSE stream and deserializes events into typed `StreamEvent` variants.
    /// Implements `futures::Stream<Item = Result<StreamEvent, Error>>`.
    pub struct MessageStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<StreamEvent, Error>> + Send>>,
    }
}

impl MessageStream {
    /// Create a new `MessageStream` from a raw reqwest Response.
    pub fn new(response: reqwest::Response) -> Self {
        let sse_stream = parse_sse_stream(response);
        let event_stream = sse_stream.map(|result| {
            match result {
                Ok(raw) => parse_stream_event(raw),
                Err(e) => Err(e),
            }
        });

        Self {
            inner: Box::pin(event_stream),
        }
    }

    /// Create a `MessageStream` from any stream of `StreamEvent` results.
    ///
    /// Useful for testing: construct a stream from a pre-built list of events
    /// without requiring an HTTP connection.
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<StreamEvent, Error>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Create a `MessageStream` from a pre-built list of events.
    ///
    /// Convenience wrapper around `from_stream` that converts a `Vec<StreamEvent>`
    /// into a stream yielding each event as `Ok(event)`.
    pub fn from_events(events: Vec<StreamEvent>) -> Self {
        Self::from_stream(futures::stream::iter(events.into_iter().map(Ok)))
    }

    /// Consume the stream and accumulate events into a final `Message`.
    ///
    /// This processes all stream events, building up the complete message
    /// by merging content block deltas into their respective blocks.
    pub async fn accumulate(self) -> Result<Message, Error> {
        self.accumulate_with(|_| {}).await
    }

    /// Consume the stream and accumulate events into a final `Message`,
    /// calling the provided callback for each event as it arrives.
    pub async fn accumulate_with(
        mut self,
        mut callback: impl FnMut(&StreamEvent),
    ) -> Result<Message, Error> {
        let mut message: Option<Message> = None;
        let mut content_blocks: Vec<ContentBlock> = Vec::new();
        // Track partial JSON for tool_use blocks (keyed by index)
        let mut partial_json_bufs: std::collections::HashMap<usize, String> =
            std::collections::HashMap::new();

        while let Some(event_result) = self.next().await {
            let event = event_result?;
            callback(&event);

            match &event {
                StreamEvent::MessageStart { message: msg } => {
                    message = Some(msg.clone());
                }
                StreamEvent::ContentBlockStart { index, content_block } => {
                    let idx = *index as usize;
                    // Ensure the vec is large enough
                    while content_blocks.len() <= idx {
                        content_blocks.push(ContentBlock::Text(crate::types::content::TextBlock {
                            text: String::new(),
                            citations: None,
                        }));
                    }
                    content_blocks[idx] = content_block.clone();
                }
                StreamEvent::ContentBlockDelta { index, delta } => {
                    let idx = *index as usize;
                    if idx < content_blocks.len() {
                        apply_delta(&mut content_blocks[idx], delta, &mut partial_json_bufs, idx);
                    }
                }
                StreamEvent::ContentBlockStop { index } => {
                    let idx = *index as usize;
                    // Finalize tool_use blocks: parse accumulated partial JSON into input
                    if let Some(json_str) = partial_json_bufs.remove(&idx)
                        && idx < content_blocks.len()
                        && let ContentBlock::ToolUse(ref mut tool_use) = content_blocks[idx]
                    {
                        tool_use.input = serde_json::from_str(&json_str)
                            .unwrap_or(serde_json::Value::String(json_str));
                    }
                }
                StreamEvent::MessageDelta { delta, usage } => {
                    if let Some(ref mut msg) = message {
                        msg.stop_reason = delta.stop_reason.clone();
                        msg.stop_sequence = delta.stop_sequence.clone();
                        msg.usage.output_tokens = usage.output_tokens;
                    }
                }
                StreamEvent::MessageStop => {
                    // Final event
                }
                StreamEvent::Ping => {
                    // Keep-alive, ignore
                }
                StreamEvent::Error { error } => {
                    return Err(Error::StreamError(format!(
                        "Stream error: {}: {}",
                        error.error_type, error.message
                    )));
                }
            }
        }

        match message {
            Some(mut msg) => {
                msg.content = content_blocks;
                Ok(msg)
            }
            None => Err(Error::StreamError(
                "Stream ended without a message_start event".to_string(),
            )),
        }
    }
}

impl Stream for MessageStream {
    type Item = Result<StreamEvent, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.as_mut().poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

/// Apply a content block delta to an existing content block.
fn apply_delta(
    block: &mut ContentBlock,
    delta: &ContentBlockDelta,
    partial_json_bufs: &mut std::collections::HashMap<usize, String>,
    index: usize,
) {
    match (block, delta) {
        (ContentBlock::Text(text_block), ContentBlockDelta::TextDelta { text }) => {
            text_block.text.push_str(text);
        }
        (ContentBlock::Thinking(thinking_block), ContentBlockDelta::ThinkingDelta { thinking }) => {
            thinking_block.thinking.push_str(thinking);
        }
        (ContentBlock::Thinking(thinking_block), ContentBlockDelta::SignatureDelta { signature }) => {
            thinking_block.signature.push_str(signature);
        }
        (ContentBlock::ToolUse(_), ContentBlockDelta::InputJsonDelta { partial_json }) => {
            partial_json_bufs
                .entry(index)
                .or_default()
                .push_str(partial_json);
        }
        _ => {
            // Other combinations (citation deltas, etc.)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stream_event_message_start() {
        let raw = RawSseEvent {
            event: Some("message_start".to_string()),
            data: Some(r#"{"message":{"id":"msg_123","type":"message","role":"assistant","content":[],"model":"claude-opus-4-6","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":10,"output_tokens":0}}}"#.to_string()),
            id: None,
            retry: None,
        };
        let event = parse_stream_event(raw).unwrap();
        match event {
            StreamEvent::MessageStart { message } => {
                assert_eq!(message.id, "msg_123");
                assert!(matches!(message.role, crate::types::common::Role::Assistant));
            }
            _ => panic!("Expected MessageStart"),
        }
    }

    #[test]
    fn test_parse_stream_event_ping() {
        let raw = RawSseEvent {
            event: Some("ping".to_string()),
            data: Some("{}".to_string()),
            id: None,
            retry: None,
        };
        let event = parse_stream_event(raw).unwrap();
        assert!(matches!(event, StreamEvent::Ping));
    }

    #[test]
    fn test_parse_stream_event_content_block_delta() {
        let raw = RawSseEvent {
            event: Some("content_block_delta".to_string()),
            data: Some(r#"{"index":0,"delta":{"type":"text_delta","text":"Hello"}}"#.to_string()),
            id: None,
            retry: None,
        };
        let event = parse_stream_event(raw).unwrap();
        match event {
            StreamEvent::ContentBlockDelta { index, delta } => {
                assert_eq!(index, 0);
                match delta {
                    ContentBlockDelta::TextDelta { text } => assert_eq!(text, "Hello"),
                    _ => panic!("Expected TextDelta"),
                }
            }
            _ => panic!("Expected ContentBlockDelta"),
        }
    }

    #[test]
    fn test_parse_stream_event_message_delta() {
        let raw = RawSseEvent {
            event: Some("message_delta".to_string()),
            data: Some(r#"{"delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":42}}"#.to_string()),
            id: None,
            retry: None,
        };
        let event = parse_stream_event(raw).unwrap();
        match event {
            StreamEvent::MessageDelta { delta, usage } => {
                assert_eq!(delta.stop_reason, Some(StopReason::EndTurn));
                assert!(delta.stop_sequence.is_none());
                assert_eq!(usage.output_tokens, 42);
            }
            _ => panic!("Expected MessageDelta"),
        }
    }

    #[test]
    fn test_apply_delta_text() {
        let mut block = ContentBlock::Text(crate::types::content::TextBlock {
            text: "Hello".to_string(),
            citations: None,
        });
        let mut bufs = std::collections::HashMap::new();
        apply_delta(
            &mut block,
            &ContentBlockDelta::TextDelta {
                text: " World".to_string(),
            },
            &mut bufs,
            0,
        );
        match block {
            ContentBlock::Text(tb) => assert_eq!(tb.text, "Hello World"),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_apply_delta_tool_use_partial_json() {
        let mut block = ContentBlock::ToolUse(crate::types::content::ToolUseBlock {
            id: "tu_1".to_string(),
            name: "get_weather".to_string(),
            input: serde_json::Value::Object(serde_json::Map::new()),
            partial_json: None,
        });
        let mut bufs = std::collections::HashMap::new();
        apply_delta(
            &mut block,
            &ContentBlockDelta::InputJsonDelta {
                partial_json: r#"{"loc"#.to_string(),
            },
            &mut bufs,
            0,
        );
        apply_delta(
            &mut block,
            &ContentBlockDelta::InputJsonDelta {
                partial_json: r#"ation":"SF"}"#.to_string(),
            },
            &mut bufs,
            0,
        );
        assert_eq!(bufs.get(&0).unwrap(), r#"{"location":"SF"}"#);
    }
}
