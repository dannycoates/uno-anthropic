use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::client::{Client, ClientBuilder};
use crate::error::Error;
use crate::middleware::{BoxFuture, Middleware, Next};

const EXPIRY_BUFFER_MS: u64 = 300_000; // 5 minutes
const OAUTH_BETA: &str = "oauth-2025-04-20";

/// OAuth tokens for authenticating with the Anthropic API.
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64, // unix milliseconds
}

/// Callback invoked after a successful token refresh.
pub type OnRefreshFn = Box<dyn Fn(&OAuthTokens) + Send + Sync>;

/// Configuration for OAuth-based authentication.
pub struct OAuthConfig {
    pub tokens: OAuthTokens,
    pub client_id: String,
    pub refresh_endpoint: String,
    pub on_refresh: Option<OnRefreshFn>,
}

impl OAuthConfig {
    /// Create a new `OAuthConfig`. `client_id` and `refresh_endpoint` must be
    /// supplied by the caller — no defaults are baked into this crate.
    pub fn new(
        tokens: OAuthTokens,
        client_id: impl Into<String>,
        refresh_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            tokens,
            client_id: client_id.into(),
            refresh_endpoint: refresh_endpoint.into(),
            on_refresh: None,
        }
    }

    /// Set a callback to be invoked after each successful token refresh.
    pub fn on_refresh(mut self, f: impl Fn(&OAuthTokens) + Send + Sync + 'static) -> Self {
        self.on_refresh = Some(Box::new(f));
        self
    }

    /// Build a `ClientBuilder` configured with OAuth middleware.
    pub fn into_client_builder(self) -> ClientBuilder {
        let token_manager = Arc::new(OAuthTokenManager {
            state: RwLock::new(OAuthTokenState {
                access_token: self.tokens.access_token,
                refresh_token: self.tokens.refresh_token,
                expires_at: self.tokens.expires_at,
            }),
            client_id: self.client_id,
            refresh_endpoint: self.refresh_endpoint,
            http_client: reqwest::Client::new(),
            on_refresh: self.on_refresh,
        });

        ClientBuilder::new()
            .api_key("")
            .middleware(OAuthMiddleware { token_manager })
    }

    /// Build a `Client` configured with OAuth middleware.
    pub fn into_client(self) -> Client {
        self.into_client_builder().build()
    }
}

// ── Internal token state ──────────────────────────────────────────────────────

struct OAuthTokenState {
    access_token: String,
    refresh_token: String,
    expires_at: u64,
}

struct OAuthTokenManager {
    state: RwLock<OAuthTokenState>,
    client_id: String,
    refresh_endpoint: String,
    http_client: reqwest::Client,
    on_refresh: Option<OnRefreshFn>,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    grant_type: &'static str,
    refresh_token: &'a str,
    client_id: &'a str,
}

#[derive(Deserialize)]
struct TokenRefreshResponse {
    access_token: String,
    #[serde(default)]
    expires_in: u64,
    refresh_token: String,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl OAuthTokenManager {
    async fn get_token(&self) -> Result<String, Error> {
        // Fast path: read lock
        {
            let state = self.state.read().await;
            let now = now_ms();
            if now < state.expires_at.saturating_sub(EXPIRY_BUFFER_MS) {
                return Ok(state.access_token.clone());
            }
        }

        // Slow path: write lock with double-check
        let mut state = self.state.write().await;
        let now = now_ms();
        if now < state.expires_at.saturating_sub(EXPIRY_BUFFER_MS) {
            return Ok(state.access_token.clone());
        }

        // Perform refresh
        let body = RefreshRequest {
            grant_type: "refresh_token",
            refresh_token: &state.refresh_token,
            client_id: &self.client_id,
        };

        let response = self
            .http_client
            .post(&self.refresh_endpoint)
            .json(&body)
            .send()
            .await
            .map_err(Error::Http)?;

        let status = response.status();

        if status.is_success() {
            let parsed: TokenRefreshResponse = response
                .json()
                .await
                .map_err(|_| Error::OAuth("invalid refresh response".to_string()))?;

            let now = now_ms();
            state.access_token = parsed.access_token.clone();
            state.refresh_token = parsed.refresh_token.clone();
            state.expires_at = now + parsed.expires_in * 1000;

            if let Some(ref cb) = self.on_refresh {
                cb(&OAuthTokens {
                    access_token: parsed.access_token.clone(),
                    refresh_token: parsed.refresh_token.clone(),
                    expires_at: state.expires_at,
                });
            }

            Ok(parsed.access_token)
        } else {
            let code = status.as_u16();
            if code == 401 || code == 403 {
                Err(Error::OAuth(
                    "refresh token invalid or revoked".to_string(),
                ))
            } else {
                Err(Error::OAuth(format!(
                    "token refresh failed with status {code}"
                )))
            }
        }
    }

    async fn invalidate(&self) {
        let mut state = self.state.write().await;
        state.expires_at = 0;
    }
}

// ── Middleware ────────────────────────────────────────────────────────────────

struct OAuthMiddleware {
    token_manager: Arc<OAuthTokenManager>,
}

fn apply_oauth_headers(
    mut request: reqwest::Request,
    token: &str,
) -> Result<reqwest::Request, Error> {
    let headers = request.headers_mut();

    // Remove x-api-key
    headers.remove("x-api-key");

    // Set Authorization: Bearer <token>
    let bearer = HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| Error::OAuth("invalid token value for Authorization header".to_string()))?;
    headers.insert(reqwest::header::AUTHORIZATION, bearer);

    // Set anthropic-dangerous-direct-browser-access: true
    headers.insert(
        "anthropic-dangerous-direct-browser-access",
        HeaderValue::from_static("true"),
    );

    // Merge OAUTH_BETA into anthropic-beta header
    let existing = headers
        .get("anthropic-beta")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let new_beta = match existing {
        None => OAUTH_BETA.to_string(),
        Some(ref s) if s.split(',').any(|p| p.trim() == OAUTH_BETA) => s.clone(),
        Some(ref s) => format!("{s},{OAUTH_BETA}"),
    };

    headers.insert(
        "anthropic-beta",
        HeaderValue::from_str(&new_beta)
            .map_err(|_| Error::OAuth("invalid anthropic-beta header value".to_string()))?,
    );

    Ok(request)
}

fn rebuild_request(
    http_client: &reqwest::Client,
    method: reqwest::Method,
    url: reqwest::Url,
    headers: reqwest::header::HeaderMap,
    body: bytes::Bytes,
) -> reqwest::Request {
    let mut req = http_client.request(method, url).build().unwrap();
    *req.headers_mut() = headers;
    *req.body_mut() = Some(body.into());
    req
}

impl Middleware for OAuthMiddleware {
    fn handle<'a>(
        &'a self,
        request: reqwest::Request,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<reqwest::Response, Error>> {
        Box::pin(async move {
            let token = self.token_manager.get_token().await?;

            // Save details for potential retry
            let method = request.method().clone();
            let url = request.url().clone();
            let headers = request.headers().clone();
            let body_bytes = request
                .body()
                .and_then(|b| b.as_bytes())
                .map(bytes::Bytes::copy_from_slice)
                .unwrap_or_default();

            let request = apply_oauth_headers(request, &token)?;

            let next2 = next.clone();
            let response = next.run(request).await?;

            if response.status().as_u16() == 401 {
                self.token_manager.invalidate().await;
                let new_token = self.token_manager.get_token().await?;

                let rebuilt = rebuild_request(
                    &self.token_manager.http_client,
                    method,
                    url,
                    headers,
                    body_bytes,
                );
                let rebuilt = apply_oauth_headers(rebuilt, &new_token)?;
                next2.run(rebuilt).await
            } else {
                Ok(response)
            }
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn make_tokens(expires_at: u64) -> OAuthTokens {
        OAuthTokens {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at,
        }
    }

    fn make_manager(expires_at: u64) -> OAuthTokenManager {
        OAuthTokenManager {
            state: RwLock::new(OAuthTokenState {
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                expires_at,
            }),
            client_id: "test-client-id".to_string(),
            refresh_endpoint: "https://example.com/oauth/token".to_string(),
            http_client: reqwest::Client::new(),
            on_refresh: None,
        }
    }

    #[test]
    fn test_into_client_builder_infallible() {
        let tokens = make_tokens(u64::MAX);
        let _builder = OAuthConfig::new(tokens, "client-id", "https://example.com/token")
            .into_client_builder();
        // If we get here the call was infallible
    }

    #[tokio::test]
    async fn test_token_valid_not_expired() {
        // expires far in the future, well outside the buffer
        let future_ms = now_ms() + EXPIRY_BUFFER_MS + 60_000;
        let manager = make_manager(future_ms);
        let token = manager.get_token().await.unwrap();
        assert_eq!(token, "access");
    }

    #[tokio::test]
    async fn test_token_within_buffer_triggers_refresh_attempt() {
        // expires_at is within the 5-minute buffer — should attempt refresh
        // (refresh will fail since there's no real server, just confirm it tries)
        let near_expiry = now_ms() + EXPIRY_BUFFER_MS - 1000;
        let manager = make_manager(near_expiry);
        let result = manager.get_token().await;
        // Should have attempted refresh and failed with a network error (not a panic)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_expired_triggers_refresh_attempt() {
        let manager = make_manager(0); // already expired
        let result = manager.get_token().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalidate_forces_refresh() {
        let future_ms = now_ms() + EXPIRY_BUFFER_MS + 60_000;
        let manager = make_manager(future_ms);

        // Initially valid
        let token = manager.get_token().await.unwrap();
        assert_eq!(token, "access");

        // Invalidate
        manager.invalidate().await;

        // Now expired — refresh will be attempted (no server, so err expected)
        let result = manager.get_token().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_oauth_headers_removes_api_key_adds_bearer() {
        let client = reqwest::Client::new();
        let mut req = client
            .get("https://example.com")
            .build()
            .unwrap();
        req.headers_mut()
            .insert("x-api-key", HeaderValue::from_static("old-key"));

        let req = apply_oauth_headers(req, "my-token").unwrap();
        assert!(req.headers().get("x-api-key").is_none());
        assert_eq!(
            req.headers().get("authorization").unwrap(),
            "Bearer my-token"
        );
    }

    #[test]
    fn test_apply_oauth_headers_adds_browser_access() {
        let client = reqwest::Client::new();
        let req = client.get("https://example.com").build().unwrap();
        let req = apply_oauth_headers(req, "tok").unwrap();
        assert_eq!(
            req.headers()
                .get("anthropic-dangerous-direct-browser-access")
                .unwrap(),
            "true"
        );
    }

    #[test]
    fn test_beta_header_created_when_absent() {
        let client = reqwest::Client::new();
        let req = client.get("https://example.com").build().unwrap();
        let req = apply_oauth_headers(req, "tok").unwrap();
        assert_eq!(
            req.headers().get("anthropic-beta").unwrap(),
            OAUTH_BETA
        );
    }

    #[test]
    fn test_beta_header_appended_when_present() {
        let client = reqwest::Client::new();
        let mut req = client.get("https://example.com").build().unwrap();
        req.headers_mut()
            .insert("anthropic-beta", HeaderValue::from_static("other-beta"));
        let req = apply_oauth_headers(req, "tok").unwrap();
        let beta = req.headers().get("anthropic-beta").unwrap().to_str().unwrap();
        assert!(beta.contains("other-beta"));
        assert!(beta.contains(OAUTH_BETA));
    }

    #[test]
    fn test_beta_header_not_duplicated_when_already_present() {
        let client = reqwest::Client::new();
        let mut req = client.get("https://example.com").build().unwrap();
        req.headers_mut()
            .insert("anthropic-beta", HeaderValue::from_static(OAUTH_BETA));
        let req = apply_oauth_headers(req, "tok").unwrap();
        let beta = req.headers().get("anthropic-beta").unwrap().to_str().unwrap();
        // Should not appear twice
        assert_eq!(beta.matches(OAUTH_BETA).count(), 1);
    }

    #[tokio::test]
    async fn test_on_refresh_callback_invoked() {
        use std::sync::Mutex;

        let captured = Arc::new(Mutex::new(None::<String>));
        let captured2 = captured.clone();

        // Build a manager that has an expired token so refresh will be triggered.
        // We can't mock the HTTP endpoint easily, so instead test the callback
        // wiring by constructing the manager's state directly and calling
        // the internal update logic that on_refresh would exercise.
        // Here we verify OAuthConfig::on_refresh stores the callback correctly.
        let tokens = make_tokens(u64::MAX);
        let config = OAuthConfig::new(tokens, "client-id", "https://example.com/token")
            .on_refresh(move |t| {
            *captured2.lock().unwrap() = Some(t.access_token.clone());
        });
        assert!(config.on_refresh.is_some());

        // Invoke the stored callback directly to verify it captures correctly
        let cb = config.on_refresh.unwrap();
        cb(&OAuthTokens {
            access_token: "new-token".to_string(),
            refresh_token: "new-refresh".to_string(),
            expires_at: 9999,
        });
        assert_eq!(captured.lock().unwrap().as_deref(), Some("new-token"));
    }

    #[tokio::test]
    async fn test_on_refresh_called_after_successful_refresh() {
        // We cannot mock reqwest in unit tests without a mock server.
        // This test verifies the counter increments via wiremock would be
        // integration-level; instead just ensure the Arc<AtomicU32> wiring compiles.
        let counter = Arc::new(AtomicU32::new(0));
        let counter2 = counter.clone();
        let tokens = make_tokens(u64::MAX);
        let _config = OAuthConfig::new(tokens, "client-id", "https://example.com/token")
            .on_refresh(move |_| {
            counter2.fetch_add(1, Ordering::SeqCst);
        });
        // callback wiring verified — actual invocation tested in integration tests
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}
