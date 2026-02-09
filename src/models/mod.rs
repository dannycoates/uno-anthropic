use serde::Serialize;

use crate::client::Client;
use crate::error::Error;
use crate::types::{ModelInfo, Page};

/// Service for the Models API.
///
/// Access via `client.models()`.
pub struct ModelService<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> ModelService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get information about a specific model.
    ///
    /// Calls `GET /v1/models/{model_id}`.
    pub async fn get(&self, model_id: &str) -> Result<ModelInfo, Error> {
        let path = format!("models/{}", model_id);
        self.client.get(&path, None).await
    }

    /// List available models.
    ///
    /// Calls `GET /v1/models` with optional pagination parameters.
    /// Returns a `Page<ModelInfo>`.
    pub async fn list(&self, params: ModelListParams) -> Result<Page<ModelInfo>, Error> {
        let query = params.to_query_string();
        let path = if query.is_empty() {
            "models".to_string()
        } else {
            format!("models?{}", query)
        };
        self.client.get(&path, None).await
    }
}

/// Parameters for listing models.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModelListParams {
    /// Maximum number of items to return per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor for pagination: return results after this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
    /// Cursor for pagination: return results before this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
}

impl ModelListParams {
    fn to_query_string(&self) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_list_params_empty() {
        let params = ModelListParams::default();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_model_list_params_with_limit() {
        let params = ModelListParams {
            limit: Some(10),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "limit=10");
    }

    #[test]
    fn test_model_list_params_with_all() {
        let params = ModelListParams {
            limit: Some(5),
            after_id: Some("model_abc".to_string()),
            before_id: Some("model_xyz".to_string()),
        };
        let qs = params.to_query_string();
        assert!(qs.contains("limit=5"));
        assert!(qs.contains("after_id=model_abc"));
        assert!(qs.contains("before_id=model_xyz"));
    }
}
