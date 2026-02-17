use reqwest::header::{HeaderMap, HeaderValue};

use crate::client::Client;
use crate::error::Error;
use crate::messages::params::{CountTokensParams, MessageCreateParams};
use crate::messages::streaming::MessageStream;
use crate::messages::{CountTokensResponse, MessageService};
use crate::types::message::Message;

// Known beta feature string constants
pub const BETA_MESSAGE_BATCHES_2024_09_24: &str = "message-batches-2024-09-24";
pub const BETA_PROMPT_CACHING_2024_07_31: &str = "prompt-caching-2024-07-31";
pub const BETA_COMPUTER_USE_2024_10_22: &str = "computer-use-2024-10-22";
pub const BETA_COMPUTER_USE_2025_01_24: &str = "computer-use-2025-01-24";
pub const BETA_PDFS_2024_09_25: &str = "pdfs-2024-09-25";
pub const BETA_TOKEN_COUNTING_2024_11_01: &str = "token-counting-2024-11-01";
pub const BETA_TOKEN_EFFICIENT_TOOLS_2025_02_19: &str = "token-efficient-tools-2025-02-19";
pub const BETA_OUTPUT_128K_2025_02_19: &str = "output-128k-2025-02-19";
pub const BETA_FILES_API_2025_04_14: &str = "files-api-2025-04-14";
pub const BETA_MCP_CLIENT_2025_04_04: &str = "mcp-client-2025-04-04";
pub const BETA_MCP_CLIENT_2025_11_20: &str = "mcp-client-2025-11-20";
pub const BETA_DEV_FULL_THINKING_2025_05_14: &str = "dev-full-thinking-2025-05-14";
pub const BETA_INTERLEAVED_THINKING_2025_05_14: &str = "interleaved-thinking-2025-05-14";
pub const BETA_CODE_EXECUTION_2025_05_22: &str = "code-execution-2025-05-22";
pub const BETA_EXTENDED_CACHE_TTL_2025_04_11: &str = "extended-cache-ttl-2025-04-11";
pub const BETA_ADAPTIVE_THINKING_2026_01_28: &str = "adaptive-thinking-2026-01-28";
pub const BETA_CLAUDE_CODE_20250219: &str = "claude-code-20250219";
pub const BETA_EFFORT_2025_11_24: &str = "effort-2025-11-24";
pub const BETA_OAUTH_2025_04_20: &str = "oauth-2025-04-20";
pub const BETA_PROMPT_CACHING_SCOPE_2026_01_05: &str = "prompt-caching-scope-2026-01-05";

/// Service for accessing beta API features.
///
/// Access via `client.beta()`.
///
/// The beta service provides access to Messages API endpoints with
/// `anthropic-beta` header injection for opting into beta features.
pub struct BetaService<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> BetaService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Access the Messages service with beta feature flags.
    ///
    /// Use `with_betas()` to specify which beta features to enable:
    /// ```ignore
    /// let msg = client.beta()
    ///     .messages()
    ///     .with_betas(vec![BETA_PROMPT_CACHING_2024_07_31.to_string()])
    ///     .create(params)
    ///     .await?;
    /// ```
    pub fn messages(&self) -> BetaMessageService<'a> {
        BetaMessageService {
            client: self.client,
            betas: Vec::new(),
        }
    }
}

/// Messages service with beta header injection.
///
/// Wraps the standard `MessageService` and injects the `anthropic-beta`
/// header with the specified beta feature strings.
pub struct BetaMessageService<'a> {
    client: &'a Client,
    betas: Vec<String>,
}

impl<'a> BetaMessageService<'a> {
    /// Set the beta features to enable for requests through this service.
    ///
    /// The beta feature strings are sent as a comma-separated list
    /// in the `anthropic-beta` header.
    pub fn with_betas(mut self, betas: Vec<String>) -> Self {
        self.betas = betas;
        self
    }

    /// Build the header map containing the anthropic-beta header.
    fn beta_headers(&self) -> Option<HeaderMap> {
        if self.betas.is_empty() {
            return None;
        }
        let mut headers = HeaderMap::new();
        let beta_value = self.betas.join(",");
        if let Ok(val) = HeaderValue::from_str(&beta_value) {
            headers.insert("anthropic-beta", val);
        }
        Some(headers)
    }

    /// Create a message (non-streaming) with beta features enabled.
    pub async fn create(&self, params: MessageCreateParams) -> Result<Message, Error> {
        let service = match self.beta_headers() {
            Some(headers) => MessageService::with_extra_headers(self.client, headers),
            None => MessageService::new(self.client),
        };
        service.create(params).await
    }

    /// Create a streaming message with beta features enabled.
    pub async fn create_stream(
        &self,
        params: MessageCreateParams,
    ) -> Result<MessageStream, Error> {
        let service = match self.beta_headers() {
            Some(headers) => MessageService::with_extra_headers(self.client, headers),
            None => MessageService::new(self.client),
        };
        service.create_stream(params).await
    }

    /// Count tokens with beta features enabled.
    pub async fn count_tokens(
        &self,
        params: CountTokensParams,
    ) -> Result<CountTokensResponse, Error> {
        let service = match self.beta_headers() {
            Some(headers) => MessageService::with_extra_headers(self.client, headers),
            None => MessageService::new(self.client),
        };
        service.count_tokens(params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_constants() {
        assert_eq!(BETA_PROMPT_CACHING_2024_07_31, "prompt-caching-2024-07-31");
        assert_eq!(BETA_COMPUTER_USE_2024_10_22, "computer-use-2024-10-22");
        assert_eq!(BETA_FILES_API_2025_04_14, "files-api-2025-04-14");
        assert_eq!(BETA_MCP_CLIENT_2025_11_20, "mcp-client-2025-11-20");
    }

    #[test]
    fn test_beta_headers_empty() {
        let client = Client::builder().api_key("test").build();
        let service = client.beta().messages();
        assert!(service.beta_headers().is_none());
    }

    #[test]
    fn test_beta_headers_single() {
        let client = Client::builder().api_key("test").build();
        let service = client
            .beta()
            .messages()
            .with_betas(vec![BETA_PROMPT_CACHING_2024_07_31.to_string()]);
        let headers = service.beta_headers().unwrap();
        assert_eq!(
            headers.get("anthropic-beta").unwrap(),
            "prompt-caching-2024-07-31"
        );
    }

    #[test]
    fn test_beta_headers_multiple() {
        let client = Client::builder().api_key("test").build();
        let service = client.beta().messages().with_betas(vec![
            BETA_PROMPT_CACHING_2024_07_31.to_string(),
            BETA_COMPUTER_USE_2024_10_22.to_string(),
        ]);
        let headers = service.beta_headers().unwrap();
        let val = headers.get("anthropic-beta").unwrap().to_str().unwrap();
        assert!(val.contains("prompt-caching-2024-07-31"));
        assert!(val.contains("computer-use-2024-10-22"));
        assert!(val.contains(","));
    }
}
