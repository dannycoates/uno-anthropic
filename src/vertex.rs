use std::sync::Arc;

use gcp_auth::TokenProvider;

use crate::client::{Client, ClientBuilder};
use crate::error::Error;
use crate::middleware::{BoxFuture, Middleware, Next};

const DEFAULT_VERTEX_VERSION: &str = "vertex-2023-10-16";

/// Configuration for Google Vertex AI integration.
///
/// Creates a `Client` pre-configured with the Vertex middleware that:
/// - Rewrites the URL to the Vertex AI endpoint format
/// - Moves the `model` field from the JSON body to the URL path
/// - Injects `anthropic_version` into the request body
/// - Adds OAuth bearer token for authentication
pub struct VertexConfig {
    region: String,
    project_id: String,
    token_provider: Arc<dyn TokenProvider>,
}

impl VertexConfig {
    /// Create a VertexConfig using Application Default Credentials.
    ///
    /// Uses `gcp_auth` to discover credentials from the environment
    /// (GOOGLE_APPLICATION_CREDENTIALS, gcloud CLI, metadata server, etc.)
    pub async fn from_env(region: impl Into<String>, project_id: impl Into<String>) -> Self {
        let provider = gcp_auth::provider()
            .await
            .expect("failed to initialize GCP auth provider");

        Self {
            region: region.into(),
            project_id: project_id.into(),
            token_provider: provider,
        }
    }

    /// Create a VertexConfig with a custom token provider.
    pub fn with_token_provider(
        region: impl Into<String>,
        project_id: impl Into<String>,
        token_provider: Arc<dyn TokenProvider>,
    ) -> Self {
        Self {
            region: region.into(),
            project_id: project_id.into(),
            token_provider,
        }
    }

    /// Build an Anthropic `Client` configured for Vertex AI.
    pub fn into_client(self) -> Client {
        self.into_client_builder().build()
    }

    /// Build an Anthropic `ClientBuilder` configured for Vertex AI.
    ///
    /// Allows further customization before building the client.
    pub fn into_client_builder(self) -> ClientBuilder {
        let base_url = if self.region == "global" {
            "https://aiplatform.googleapis.com".to_string()
        } else {
            format!("https://{}-aiplatform.googleapis.com", self.region)
        };

        Client::builder()
            .base_url(base_url)
            .api_key("") // Vertex uses OAuth, not API keys
            .middleware(VertexMiddleware {
                region: self.region,
                project_id: self.project_id,
                token_provider: self.token_provider,
            })
    }
}

/// Middleware that transforms requests for Google Vertex AI compatibility.
struct VertexMiddleware {
    region: String,
    project_id: String,
    token_provider: Arc<dyn TokenProvider>,
}

impl Middleware for VertexMiddleware {
    fn handle<'a>(
        &'a self,
        request: reqwest::Request,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<reqwest::Response, Error>> {
        Box::pin(async move {
            let mut request = request;

            // Get OAuth token
            let token = self
                .token_provider
                .token(&["https://www.googleapis.com/auth/cloud-platform"])
                .await
                .map_err(|e| Error::StreamError(format!("Failed to get GCP token: {}", e)))?;

            // Set Authorization header
            let auth_value = format!("Bearer {}", token.as_str());
            request.headers_mut().insert(
                reqwest::header::AUTHORIZATION,
                auth_value
                    .parse()
                    .map_err(|e| Error::StreamError(format!("Invalid auth header: {}", e)))?,
            );

            // Remove x-api-key header (Vertex uses OAuth)
            request.headers_mut().remove("x-api-key");

            // Read and transform the body
            if let Some(body_bytes) = request.body().and_then(|b| b.as_bytes()) {
                let mut body: serde_json::Value =
                    serde_json::from_slice(body_bytes).unwrap_or(serde_json::Value::Null);

                if let Some(obj) = body.as_object_mut() {
                    // Inject anthropic_version if not present
                    if !obj.contains_key("anthropic_version") {
                        obj.insert(
                            "anthropic_version".to_string(),
                            serde_json::Value::String(DEFAULT_VERTEX_VERSION.to_string()),
                        );
                    }

                    let path = request.url().path().to_string();
                    let method = request.method().clone();

                    // Rewrite /v1/messages endpoint
                    if path.ends_with("/messages") && method == reqwest::Method::POST {
                        let model = obj
                            .remove("model")
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                            .unwrap_or_default();

                        let stream = obj
                            .get("stream")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        let specifier = if stream {
                            "streamRawPredict"
                        } else {
                            "rawPredict"
                        };

                        let new_path = format!(
                            "/v1/projects/{}/locations/{}/publishers/anthropic/models/{}:{}",
                            self.project_id, self.region, model, specifier
                        );
                        let mut url = request.url().clone();
                        url.set_path(&new_path);
                        *request.url_mut() = url;
                    }

                    // Rewrite /v1/messages/count_tokens endpoint
                    if path.ends_with("/messages/count_tokens") && method == reqwest::Method::POST {
                        let new_path = format!(
                            "/v1/projects/{}/locations/{}/publishers/anthropic/models/count-tokens:rawPredict",
                            self.project_id, self.region
                        );
                        let mut url = request.url().clone();
                        url.set_path(&new_path);
                        *request.url_mut() = url;
                    }
                }

                // Set the modified body
                let new_body =
                    serde_json::to_vec(&body).map_err(Error::Serialization)?;
                *request.body_mut() = Some(reqwest::Body::from(new_body));
            }

            next.run(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_vertex_version() {
        assert_eq!(DEFAULT_VERTEX_VERSION, "vertex-2023-10-16");
    }
}
