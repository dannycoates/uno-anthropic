use std::sync::Arc;
use std::time::Duration;

use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, warn};

use crate::config::ClientConfig;
use crate::error::{ApiErrorResponse, Error, is_retryable_status};
use crate::middleware::{BoxFuture, Middleware, execute_middleware_chain};
use crate::retry::{RetryPolicy, check_should_retry_header, parse_retry_after};

/// Shared inner state for the client.
pub(crate) struct ClientInner {
    pub(crate) http: reqwest::Client,
    pub(crate) config: ClientConfig,
    pub(crate) retry_policy: RetryPolicy,
    pub(crate) middlewares: Vec<Box<dyn Middleware>>,
}

/// The Anthropic API client.
///
/// Holds an `Arc<ClientInner>` for cheap cloning. Services borrow `&Client`.
#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.inner.config.base_url)
            .field("max_retries", &self.inner.retry_policy.max_retries)
            .finish()
    }
}

impl Client {
    /// Create a new client with default configuration from environment variables.
    ///
    /// Reads `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL` from the environment.
    pub fn new() -> Self {
        ClientBuilder::new().build()
    }

    /// Create a new `ClientBuilder` for customizing client configuration.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Access the Messages service.
    pub fn messages(&self) -> crate::messages::MessageService<'_> {
        crate::messages::MessageService::new(self)
    }

    /// Access the Models service.
    pub fn models(&self) -> crate::models::ModelService<'_> {
        crate::models::ModelService::new(self)
    }

    /// Access the Batches service.
    pub fn batches(&self) -> crate::batches::BatchService<'_> {
        crate::batches::BatchService::new(self)
    }

    /// Access the Beta service.
    pub fn beta(&self) -> crate::beta::BetaService<'_> {
        crate::beta::BetaService::new(self)
    }

    /// Execute a POST request, deserializing the JSON response into `T`.
    ///
    /// Handles middleware chain execution, retry logic, and error parsing.
    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
        extra_headers: Option<&HeaderMap>,
    ) -> Result<T, Error> {
        let bytes = self.execute_raw("POST", path, Some(body), extra_headers).await?;
        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    /// Execute a GET request, deserializing the JSON response into `T`.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        extra_headers: Option<&HeaderMap>,
    ) -> Result<T, Error> {
        let bytes = self.execute_raw("GET", path, None::<&()>, extra_headers).await?;
        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    /// Execute a DELETE request, deserializing the JSON response into `T`.
    pub(crate) async fn delete<T: DeserializeOwned>(
        &self,
        path: &str,
        extra_headers: Option<&HeaderMap>,
    ) -> Result<T, Error> {
        let bytes = self.execute_raw("DELETE", path, None::<&()>, extra_headers).await?;
        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    /// Execute a raw HTTP request with retry logic and middleware.
    ///
    /// Returns the raw response bytes on success.
    pub(crate) async fn execute_raw<B: Serialize>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        extra_headers: Option<&HeaderMap>,
    ) -> Result<bytes::Bytes, Error> {
        let inner = &self.inner;
        let url = format!("{}/v1/{}", inner.config.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        let headers = inner.config.build_headers();

        let max_retries = inner.retry_policy.max_retries;

        for attempt in 0..=max_retries {
            let mut request = inner.http.request(
                method.parse().unwrap_or(reqwest::Method::GET),
                &url,
            );

            request = request.headers(headers.clone());

            if let Some(extra) = extra_headers {
                request = request.headers(extra.clone());
            }

            if let Some(b) = body {
                request = request.json(b);
            }

            let req = request.build().map_err(Error::Http)?;

            debug!(attempt, url = %url, method, "executing request");

            let result = if inner.middlewares.is_empty() {
                inner.http.execute(req).await.map_err(Error::Http)
            } else {
                let http = &inner.http;
                execute_middleware_chain(
                    &inner.middlewares,
                    req,
                    move |r| -> BoxFuture<'_, Result<reqwest::Response, Error>> {
                        Box::pin(async move {
                            http.execute(r).await.map_err(Error::Http)
                        })
                    },
                )
                .await
            };

            match result {
                Ok(response) => {
                    let status = response.status().as_u16();

                    if status >= 400 {
                        // Check x-should-retry header
                        let should_retry = check_should_retry_header(response.headers());
                        let retry_after = parse_retry_after(response.headers());
                        let retryable = should_retry.unwrap_or_else(|| is_retryable_status(status));

                        // Try to parse the error body
                        let body_bytes = response.bytes().await.map_err(Error::Http)?;
                        let error_body = serde_json::from_slice::<ApiErrorResponse>(&body_bytes)
                            .map(|r| r.error)
                            .unwrap_or_else(|_| crate::error::ApiErrorBody {
                                error_type: "unknown_error".to_string(),
                                message: String::from_utf8_lossy(&body_bytes).to_string(),
                            });

                        if retryable && attempt < max_retries {
                            let delay = inner.retry_policy.delay_for_attempt(attempt, retry_after);
                            warn!(
                                attempt,
                                status,
                                delay_ms = delay.as_millis() as u64,
                                "retrying request"
                            );
                            tokio::time::sleep(delay).await;
                            continue;
                        }

                        return Err(Error::Api {
                            status,
                            body: error_body,
                        });
                    }

                    let bytes = response.bytes().await.map_err(Error::Http)?;
                    return Ok(bytes);
                }
                Err(e) => {
                    if e.is_retryable() && attempt < max_retries {
                        let delay = inner.retry_policy.delay_for_attempt(attempt, None);
                        warn!(
                            attempt,
                            error = %e,
                            delay_ms = delay.as_millis() as u64,
                            "retrying after error"
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        unreachable!("retry loop should always return")
    }

    /// Execute a streaming POST request, returning the raw response for SSE parsing.
    ///
    /// Injects `"stream": true` into the serialized JSON body.
    pub(crate) async fn execute_streaming(
        &self,
        path: &str,
        body: &impl Serialize,
        extra_headers: Option<&HeaderMap>,
    ) -> Result<reqwest::Response, Error> {
        let inner = &self.inner;
        let url = format!("{}/v1/{}", inner.config.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        let headers = inner.config.build_headers();

        // Serialize to Value and inject "stream": true
        let mut body_value = serde_json::to_value(body)?;
        if let Some(obj) = body_value.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }

        let max_retries = inner.retry_policy.max_retries;

        for attempt in 0..=max_retries {
            let mut request = inner.http.request(reqwest::Method::POST, &url);
            request = request.headers(headers.clone());

            if let Some(extra) = extra_headers {
                request = request.headers(extra.clone());
            }

            request = request.json(&body_value);

            let req = request.build().map_err(Error::Http)?;

            debug!(attempt, url = %url, "executing streaming request");

            let result = if inner.middlewares.is_empty() {
                inner.http.execute(req).await.map_err(Error::Http)
            } else {
                let http = &inner.http;
                execute_middleware_chain(
                    &inner.middlewares,
                    req,
                    move |r| -> BoxFuture<'_, Result<reqwest::Response, Error>> {
                        Box::pin(async move {
                            http.execute(r).await.map_err(Error::Http)
                        })
                    },
                )
                .await
            };

            match result {
                Ok(response) => {
                    let status = response.status().as_u16();

                    if status >= 400 {
                        let should_retry = check_should_retry_header(response.headers());
                        let retry_after = parse_retry_after(response.headers());
                        let retryable = should_retry.unwrap_or_else(|| is_retryable_status(status));

                        let body_bytes = response.bytes().await.map_err(Error::Http)?;
                        let error_body = serde_json::from_slice::<ApiErrorResponse>(&body_bytes)
                            .map(|r| r.error)
                            .unwrap_or_else(|_| crate::error::ApiErrorBody {
                                error_type: "unknown_error".to_string(),
                                message: String::from_utf8_lossy(&body_bytes).to_string(),
                            });

                        if retryable && attempt < max_retries {
                            let delay = inner.retry_policy.delay_for_attempt(attempt, retry_after);
                            warn!(
                                attempt,
                                status,
                                delay_ms = delay.as_millis() as u64,
                                "retrying streaming request"
                            );
                            tokio::time::sleep(delay).await;
                            continue;
                        }

                        return Err(Error::Api {
                            status,
                            body: error_body,
                        });
                    }

                    return Ok(response);
                }
                Err(e) => {
                    if e.is_retryable() && attempt < max_retries {
                        let delay = inner.retry_policy.delay_for_attempt(attempt, None);
                        warn!(
                            attempt,
                            error = %e,
                            delay_ms = delay.as_millis() as u64,
                            "retrying streaming request after error"
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        unreachable!("retry loop should always return")
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing a `Client` with custom configuration.
pub struct ClientBuilder {
    config: ClientConfig,
    retry_policy: RetryPolicy,
    http_client: Option<reqwest::Client>,
    middlewares: Vec<Box<dyn Middleware>>,
    proxy_url: Option<String>,
    accept_invalid_certs: bool,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            config: ClientConfig::from_env(),
            retry_policy: RetryPolicy::default(),
            http_client: None,
            middlewares: Vec::new(),
            proxy_url: None,
            accept_invalid_certs: false,
        }
    }

    /// Set the API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.api_key = key.into();
        self
    }

    /// Set the base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }

    /// Set the maximum number of retries.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.retry_policy.max_retries = retries;
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set a custom reqwest HTTP client.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Set the User-Agent string.
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.config.user_agent = ua.into();
        self
    }

    /// Set the beta features to enable via the `anthropic-beta` header.
    pub fn beta_features(mut self, betas: Vec<String>) -> Self {
        self.config.beta_features = betas;
        self
    }

    /// Add a default header.
    pub fn default_header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(name), Ok(value)) = (
            name.parse::<reqwest::header::HeaderName>(),
            value.parse::<reqwest::header::HeaderValue>(),
        ) {
            self.config.default_headers.insert(name, value);
        }
        self
    }

    /// Add a middleware to the chain.
    pub fn middleware(mut self, m: impl Middleware + 'static) -> Self {
        self.middlewares.push(Box::new(m));
        self
    }

    /// Route all requests through the given proxy URL.
    ///
    /// Ignored if a custom `http_client` is provided.
    pub fn proxy_url(mut self, url: impl Into<String>) -> Self {
        self.proxy_url = Some(url.into());
        self
    }

    /// Disable TLS certificate verification.
    ///
    /// **Use only in test environments** (e.g. mitmproxy with a self-signed cert).
    /// Ignored if a custom `http_client` is provided.
    pub fn danger_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Build the `Client`.
    pub fn build(self) -> Client {
        let http = self.http_client.unwrap_or_else(|| {
            let mut builder = reqwest::Client::builder().timeout(self.config.timeout);

            if let Some(ref proxy_url) = self.proxy_url {
                builder = builder.proxy(
                    reqwest::Proxy::all(proxy_url).expect("invalid proxy URL"),
                );
            }
            if self.accept_invalid_certs {
                builder = builder.danger_accept_invalid_certs(true);
            }

            builder.build().expect("failed to build reqwest client")
        });

        Client {
            inner: Arc::new(ClientInner {
                http,
                config: self.config,
                retry_policy: self.retry_policy,
                middlewares: self.middlewares,
            }),
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder_defaults() {
        let client = ClientBuilder::new()
            .api_key("test-key")
            .build();
        assert_eq!(client.inner.config.api_key, "test-key");
        assert_eq!(client.inner.config.base_url, "https://api.anthropic.com");
        assert_eq!(client.inner.retry_policy.max_retries, 2);
    }

    #[test]
    fn test_client_builder_custom() {
        let client = ClientBuilder::new()
            .api_key("my-key")
            .base_url("https://custom.api.com")
            .max_retries(5)
            .timeout(Duration::from_secs(30))
            .default_header("x-custom", "value")
            .build();

        assert_eq!(client.inner.config.api_key, "my-key");
        assert_eq!(client.inner.config.base_url, "https://custom.api.com");
        assert_eq!(client.inner.retry_policy.max_retries, 5);
        assert_eq!(client.inner.config.timeout, Duration::from_secs(30));
        assert_eq!(
            client.inner.config.default_headers.get("x-custom").unwrap(),
            "value"
        );
    }

    #[test]
    fn test_client_clone_is_cheap() {
        let client = Client::builder().api_key("key").build();
        let cloned = client.clone();
        assert!(Arc::ptr_eq(&client.inner, &cloned.inner));
    }

    #[test]
    fn test_client_builder_proxy() {
        let client = ClientBuilder::new()
            .api_key("test-key")
            .proxy_url("http://127.0.0.1:8080")
            .danger_accept_invalid_certs(true)
            .build();
        // Proxy and cert settings are applied during build; verify the client was constructed.
        assert_eq!(client.inner.config.api_key, "test-key");
    }

    #[test]
    fn test_client_debug() {
        let client = Client::builder()
            .api_key("key")
            .base_url("https://api.example.com")
            .build();
        let debug = format!("{:?}", client);
        assert!(debug.contains("https://api.example.com"));
    }
}
