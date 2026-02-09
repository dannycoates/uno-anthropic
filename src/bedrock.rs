use std::time::SystemTime;

use aws_credential_types::provider::ProvideCredentials;
use aws_sigv4::http_request::{
    PayloadChecksumKind, SignableBody, SignableRequest, SignatureLocation, SigningSettings,
    sign as sigv4_sign,
};
use aws_sigv4::sign::v4;
use aws_smithy_runtime_api::client::identity::Identity;
use reqwest::header::HeaderValue;

use crate::client::{Client, ClientBuilder};
use crate::error::Error;
use crate::middleware::{BoxFuture, Middleware, Next};

const DEFAULT_BEDROCK_VERSION: &str = "bedrock-2023-05-31";

/// Configuration for AWS Bedrock integration.
///
/// Creates a `Client` pre-configured with the Bedrock middleware that:
/// - Rewrites the URL to the Bedrock endpoint format
/// - Moves the `model` field from the JSON body to the URL path
/// - Injects `anthropic_version` into the request body
/// - Signs requests with AWS SigV4
pub struct BedrockConfig {
    region: String,
    credentials_provider: Box<dyn ProvideCredentials>,
}

impl BedrockConfig {
    /// Create a BedrockConfig from environment variables.
    ///
    /// Uses the default AWS credential chain (env vars, config files, IMDS, etc.)
    /// via `aws-config`.
    pub async fn from_env(region: impl Into<String>) -> Self {
        let region_str = region.into();
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region_str.clone()))
            .load()
            .await;

        let provider = aws_config
            .credentials_provider()
            .expect("no AWS credentials provider found")
            .clone();

        Self {
            region: region_str,
            credentials_provider: Box::new(provider),
        }
    }

    /// Create a BedrockConfig with a custom credentials provider.
    pub fn with_credentials(
        region: impl Into<String>,
        credentials_provider: impl ProvideCredentials + 'static,
    ) -> Self {
        Self {
            region: region.into(),
            credentials_provider: Box::new(credentials_provider),
        }
    }

    /// Build an Anthropic `Client` configured for Bedrock.
    pub fn into_client(self) -> Client {
        self.into_client_builder().build()
    }

    /// Build an Anthropic `ClientBuilder` configured for Bedrock.
    ///
    /// Allows further customization before building the client.
    pub fn into_client_builder(self) -> ClientBuilder {
        let base_url = format!("https://bedrock-runtime.{}.amazonaws.com", self.region);
        Client::builder()
            .base_url(base_url)
            .api_key("") // Bedrock uses SigV4, not API keys
            .middleware(BedrockMiddleware {
                region: self.region,
                credentials_provider: self.credentials_provider,
            })
    }
}

/// Middleware that transforms requests for AWS Bedrock compatibility.
struct BedrockMiddleware {
    region: String,
    credentials_provider: Box<dyn ProvideCredentials>,
}

impl Middleware for BedrockMiddleware {
    fn handle<'a>(
        &'a self,
        request: reqwest::Request,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<reqwest::Response, Error>> {
        Box::pin(async move {
            let mut request = request;

            // Read and transform the body
            if let Some(body_bytes) = request.body().and_then(|b| b.as_bytes()) {
                let mut body: serde_json::Value =
                    serde_json::from_slice(body_bytes).unwrap_or(serde_json::Value::Null);

                if let Some(obj) = body.as_object_mut() {
                    // Inject anthropic_version if not present
                    if !obj.contains_key("anthropic_version") {
                        obj.insert(
                            "anthropic_version".to_string(),
                            serde_json::Value::String(DEFAULT_BEDROCK_VERSION.to_string()),
                        );
                    }

                    // For POST requests to messages or complete endpoints, transform the URL
                    let path = request.url().path().to_string();
                    let method = request.method().clone();

                    if method == reqwest::Method::POST
                        && (path.ends_with("/messages") || path.ends_with("/complete"))
                    {
                        // Extract model from body
                        let model = obj
                            .remove("model")
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                            .unwrap_or_default();

                        // Determine invoke method based on stream field
                        let stream = obj
                            .get("stream")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        obj.remove("stream");

                        let invoke_method = if stream {
                            "invoke-with-response-stream"
                        } else {
                            "invoke"
                        };

                        // Rewrite URL path
                        let new_path = format!("/model/{}/{}", model, invoke_method);
                        let mut url = request.url().clone();
                        url.set_path(&new_path);
                        *request.url_mut() = url;
                    }
                }

                // Set the modified body
                let new_body =
                    serde_json::to_vec(&body).map_err(Error::Serialization)?;
                *request.body_mut() = Some(reqwest::Body::from(new_body.clone()));

                // Remove x-api-key header (Bedrock uses SigV4)
                request.headers_mut().remove("x-api-key");

                // Get AWS credentials and convert to Identity for SigV4
                let credentials = self
                    .credentials_provider
                    .provide_credentials()
                    .await
                    .map_err(|e| {
                        Error::StreamError(format!("Failed to get AWS credentials: {}", e))
                    })?;

                let identity: Identity = credentials.into();

                let mut signing_settings = SigningSettings::default();
                signing_settings.payload_checksum_kind = PayloadChecksumKind::XAmzSha256;
                signing_settings.signature_location = SignatureLocation::Headers;

                let signing_params = v4::SigningParams::builder()
                    .identity(&identity)
                    .region(&self.region)
                    .name("bedrock")
                    .time(SystemTime::now())
                    .settings(signing_settings)
                    .build()
                    .map_err(|e| {
                        Error::StreamError(format!("Failed to build signing params: {}", e))
                    })?;

                let signable_request = SignableRequest::new(
                    request.method().as_str(),
                    request.url().as_str(),
                    request
                        .headers()
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or(""))),
                    SignableBody::Bytes(&new_body),
                )
                .map_err(|e| {
                    Error::StreamError(format!("Failed to create signable request: {}", e))
                })?;

                let (signing_instructions, _signature) =
                    sigv4_sign(signable_request, &signing_params.into())
                        .map_err(|e| Error::StreamError(format!("SigV4 signing failed: {}", e)))?
                        .into_parts();

                // Apply signing headers
                for (name, value) in signing_instructions.headers() {
                    let header_name: reqwest::header::HeaderName =
                        name.parse().map_err(|e| {
                            Error::StreamError(format!("Invalid header name: {}", e))
                        })?;
                    let header_value = HeaderValue::from_str(value).map_err(|e| {
                        Error::StreamError(format!("Invalid header value: {}", e))
                    })?;
                    request.headers_mut().insert(header_name, header_value);
                }
            }

            next.run(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bedrock_version() {
        assert_eq!(DEFAULT_BEDROCK_VERSION, "bedrock-2023-05-31");
    }
}
