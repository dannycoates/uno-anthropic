pub mod params;
pub mod streaming;

use reqwest::header::HeaderMap;
use serde::Deserialize;

use crate::client::Client;
use crate::error::Error;
use crate::types::message::Message;

use self::params::{CountTokensParams, MessageCreateParams};
use self::streaming::MessageStream;

/// Response from the count_tokens endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CountTokensResponse {
    pub input_tokens: u32,
}

/// Service for the Messages API.
pub struct MessageService<'a> {
    pub(crate) client: &'a Client,
    pub(crate) extra_headers: Option<HeaderMap>,
}

impl<'a> MessageService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            extra_headers: None,
        }
    }

    pub(crate) fn with_extra_headers(client: &'a Client, headers: HeaderMap) -> Self {
        Self {
            client,
            extra_headers: Some(headers),
        }
    }

    /// Create a message (non-streaming).
    ///
    /// Sends a POST request to `/v1/messages` with `"stream": false` injected.
    pub async fn create(&self, params: MessageCreateParams) -> Result<Message, Error> {
        // Serialize to Value and inject "stream": false
        let mut body = serde_json::to_value(&params)?;
        if let Some(obj) = body.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(false));
        }

        self.client
            .post("messages", &body, self.extra_headers.as_ref())
            .await
    }

    /// Create a streaming message.
    ///
    /// Sends a POST request to `/v1/messages` with `"stream": true` injected.
    /// Returns a `MessageStream` that yields `StreamEvent` items.
    pub async fn create_stream(
        &self,
        params: MessageCreateParams,
    ) -> Result<MessageStream, Error> {
        let response = self
            .client
            .execute_streaming("messages", &params, self.extra_headers.as_ref())
            .await?;

        Ok(MessageStream::new(response))
    }

    /// Count the tokens in a set of messages.
    ///
    /// Sends a POST request to `/v1/messages/count_tokens`.
    pub async fn count_tokens(
        &self,
        params: CountTokensParams,
    ) -> Result<CountTokensResponse, Error> {
        self.client
            .post("messages/count_tokens", &params, self.extra_headers.as_ref())
            .await
    }
}
