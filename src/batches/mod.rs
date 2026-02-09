pub mod types;

use std::pin::Pin;

use futures::stream::Stream;

use crate::client::Client;
use crate::error::Error;
use crate::types::Page;

pub use self::types::*;

/// Service for the Message Batches API.
///
/// Access via `client.batches()`.
pub struct BatchService<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> BatchService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new message batch.
    ///
    /// Calls `POST /v1/messages/batches`.
    pub async fn create(&self, params: BatchCreateParams) -> Result<MessageBatch, Error> {
        self.client.post("messages/batches", &params, None).await
    }

    /// Get a message batch by ID.
    ///
    /// Calls `GET /v1/messages/batches/{batch_id}`.
    pub async fn get(&self, batch_id: &str) -> Result<MessageBatch, Error> {
        let path = format!("messages/batches/{}", batch_id);
        self.client.get(&path, None).await
    }

    /// List message batches.
    ///
    /// Calls `GET /v1/messages/batches` with optional pagination parameters.
    pub async fn list(&self, params: BatchListParams) -> Result<Page<MessageBatch>, Error> {
        let query = params.to_query_string();
        let path = if query.is_empty() {
            "messages/batches".to_string()
        } else {
            format!("messages/batches?{}", query)
        };
        self.client.get(&path, None).await
    }

    /// Cancel a message batch.
    ///
    /// Calls `POST /v1/messages/batches/{batch_id}/cancel`.
    pub async fn cancel(&self, batch_id: &str) -> Result<MessageBatch, Error> {
        let path = format!("messages/batches/{}/cancel", batch_id);
        self.client
            .post::<MessageBatch>(&path, &serde_json::Value::Null, None)
            .await
    }

    /// Delete a message batch.
    ///
    /// Calls `DELETE /v1/messages/batches/{batch_id}`.
    pub async fn delete(&self, batch_id: &str) -> Result<DeletedMessageBatch, Error> {
        let path = format!("messages/batches/{}", batch_id);
        self.client.delete(&path, None).await
    }

    /// Stream the results of a completed message batch as JSONL.
    ///
    /// Calls `GET /v1/messages/batches/{batch_id}/results`.
    /// Returns a stream of `BatchResult` items parsed from JSONL.
    pub async fn results(
        &self,
        batch_id: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<BatchResult, Error>> + Send>>, Error> {
        let path = format!("messages/batches/{}/results", batch_id);

        // Execute a raw GET and get the response body as a byte stream
        let bytes = self.client.execute_raw("GET", &path, None::<&()>, None).await?;

        // Parse JSONL: each line is a JSON object
        let lines = String::from_utf8_lossy(&bytes).to_string();
        let results: Vec<Result<BatchResult, Error>> = lines
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<BatchResult>(line)
                    .map_err(|e| Error::StreamError(format!("Failed to parse batch result: {}", e)))
            })
            .collect();

        Ok(Box::pin(futures::stream::iter(results)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_list_params_default() {
        let params = BatchListParams::default();
        assert_eq!(params.to_query_string(), "");
    }
}
