use serde::Deserialize;

/// Token usage information returned with a message response.
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(default)]
    pub cache_read_input_tokens: Option<u32>,
    #[serde(default)]
    pub server_tool_use: Option<ServerToolUsage>,
}

/// Usage information specific to server tool use.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerToolUsage {
    #[serde(default)]
    pub web_search_requests: Option<u32>,
}

/// Usage information in a `message_delta` streaming event.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaUsage {
    pub output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_usage() {
        let json = r#"{
            "input_tokens": 100,
            "output_tokens": 50
        }"#;
        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert!(usage.cache_creation_input_tokens.is_none());
        assert!(usage.cache_read_input_tokens.is_none());
        assert!(usage.server_tool_use.is_none());
    }

    #[test]
    fn test_deserialize_usage_with_cache() {
        let json = r#"{
            "input_tokens": 100,
            "output_tokens": 50,
            "cache_creation_input_tokens": 200,
            "cache_read_input_tokens": 150
        }"#;
        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.cache_creation_input_tokens, Some(200));
        assert_eq!(usage.cache_read_input_tokens, Some(150));
    }

    #[test]
    fn test_deserialize_usage_with_server_tool_use() {
        let json = r#"{
            "input_tokens": 100,
            "output_tokens": 50,
            "server_tool_use": {
                "web_search_requests": 3
            }
        }"#;
        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(
            usage.server_tool_use.as_ref().unwrap().web_search_requests,
            Some(3)
        );
    }

    #[test]
    fn test_deserialize_message_delta_usage() {
        let json = r#"{"output_tokens": 42}"#;
        let usage: MessageDeltaUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.output_tokens, 42);
    }
}
