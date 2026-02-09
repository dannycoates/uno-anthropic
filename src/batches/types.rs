use serde::{Deserialize, Serialize};

use crate::error::ApiErrorBody;
use crate::messages::params::MessageCreateParams;
use crate::types::message::Message;

/// A message batch returned by the Batches API.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageBatch {
    pub id: String,
    #[serde(rename = "type")]
    pub batch_type: String,
    pub processing_status: BatchProcessingStatus,
    pub request_counts: BatchRequestCounts,
    #[serde(default)]
    pub ended_at: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub cancel_initiated_at: Option<String>,
    #[serde(default)]
    pub results_url: Option<String>,
}

/// Processing status of a message batch.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum BatchProcessingStatus {
    InProgress,
    Canceling,
    Ended,
}

/// Counts of requests in a batch, categorized by status.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchRequestCounts {
    pub processing: u32,
    pub succeeded: u32,
    pub errored: u32,
    pub canceled: u32,
    pub expired: u32,
}

/// Parameters for creating a message batch.
#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateParams {
    pub requests: Vec<BatchMessageRequest>,
}

/// A single request within a batch.
#[derive(Debug, Clone, Serialize)]
pub struct BatchMessageRequest {
    pub custom_id: String,
    pub params: MessageCreateParams,
}

/// Parameters for listing batches.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BatchListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
}

impl BatchListParams {
    pub(crate) fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(limit) = self.limit {
            parts.push(format!("limit={}", limit));
        }
        if let Some(ref after_id) = self.after_id {
            parts.push(format!("after_id={}", after_id));
        }
        if let Some(ref before_id) = self.before_id {
            parts.push(format!("before_id={}", before_id));
        }
        parts.join("&")
    }
}

/// A single result line from the batch results JSONL file.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchResult {
    pub custom_id: String,
    pub result: BatchResultBody,
}

/// The result body for a batch request.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BatchResultBody {
    Succeeded { message: Message },
    Errored { error: ApiErrorBody },
    Canceled,
    Expired,
}

/// Response from deleting a message batch.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedMessageBatch {
    pub id: String,
    #[serde(rename = "type")]
    pub deleted_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_message_batch() {
        let json = r#"{
            "id": "msgbatch_123",
            "type": "message_batch",
            "processing_status": "in_progress",
            "request_counts": {
                "processing": 5,
                "succeeded": 0,
                "errored": 0,
                "canceled": 0,
                "expired": 0
            },
            "created_at": "2025-01-01T00:00:00Z",
            "expires_at": "2025-01-02T00:00:00Z"
        }"#;
        let batch: MessageBatch = serde_json::from_str(json).unwrap();
        assert_eq!(batch.id, "msgbatch_123");
        assert_eq!(batch.batch_type, "message_batch");
        assert!(matches!(
            batch.processing_status,
            BatchProcessingStatus::InProgress
        ));
        assert_eq!(batch.request_counts.processing, 5);
        assert_eq!(batch.created_at, "2025-01-01T00:00:00Z");
        assert!(batch.ended_at.is_none());
    }

    #[test]
    fn test_deserialize_batch_result_succeeded() {
        let json = r#"{
            "custom_id": "req_1",
            "result": {
                "type": "succeeded",
                "message": {
                    "id": "msg_abc",
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "text", "text": "Hello"}],
                    "model": "claude-opus-4-6",
                    "stop_reason": "end_turn",
                    "usage": {"input_tokens": 10, "output_tokens": 5}
                }
            }
        }"#;
        let result: BatchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.custom_id, "req_1");
        match result.result {
            BatchResultBody::Succeeded { message } => {
                assert_eq!(message.id, "msg_abc");
            }
            _ => panic!("Expected Succeeded"),
        }
    }

    #[test]
    fn test_deserialize_batch_result_errored() {
        let json = r#"{
            "custom_id": "req_2",
            "result": {
                "type": "errored",
                "error": {
                    "type": "invalid_request_error",
                    "message": "Bad request"
                }
            }
        }"#;
        let result: BatchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.custom_id, "req_2");
        match result.result {
            BatchResultBody::Errored { error } => {
                assert_eq!(error.error_type, "invalid_request_error");
            }
            _ => panic!("Expected Errored"),
        }
    }

    #[test]
    fn test_deserialize_batch_result_canceled() {
        let json = r#"{
            "custom_id": "req_3",
            "result": {
                "type": "canceled"
            }
        }"#;
        let result: BatchResult = serde_json::from_str(json).unwrap();
        assert!(matches!(result.result, BatchResultBody::Canceled));
    }

    #[test]
    fn test_deserialize_batch_result_expired() {
        let json = r#"{
            "custom_id": "req_4",
            "result": {
                "type": "expired"
            }
        }"#;
        let result: BatchResult = serde_json::from_str(json).unwrap();
        assert!(matches!(result.result, BatchResultBody::Expired));
    }

    #[test]
    fn test_deserialize_deleted_batch() {
        let json = r#"{
            "id": "msgbatch_123",
            "type": "message_batch_deleted"
        }"#;
        let deleted: DeletedMessageBatch = serde_json::from_str(json).unwrap();
        assert_eq!(deleted.id, "msgbatch_123");
        assert_eq!(deleted.deleted_type, "message_batch_deleted");
    }

    #[test]
    fn test_batch_list_params_query_string() {
        let params = BatchListParams {
            limit: Some(10),
            after_id: Some("batch_abc".to_string()),
            before_id: None,
        };
        let qs = params.to_query_string();
        assert!(qs.contains("limit=10"));
        assert!(qs.contains("after_id=batch_abc"));
    }
}
