use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_TIMEOUT_SECS: u64 = 600;
const DEFAULT_MAX_RETRIES: u32 = 2;
pub const DEFAULT_USER_AGENT: &str = "Anthropic/Rust 0.1.0";

/// Configuration for the Anthropic API client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub max_retries: u32,
    pub timeout: Duration,
    pub default_headers: HeaderMap,
    pub user_agent: String,
    pub beta_features: Vec<String>,
}

impl ClientConfig {
    /// Create a new ClientConfig from environment variables and defaults.
    ///
    /// Reads `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL` from the environment.
    /// Falls back to the default base URL if `ANTHROPIC_BASE_URL` is not set.
    pub fn from_env() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let base_url =
            std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        Self {
            api_key,
            base_url,
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            default_headers: HeaderMap::new(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            beta_features: Vec::new(),
        }
    }

    /// Build the full set of default headers for requests.
    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert("anthropic-version", HeaderValue::from_static(DEFAULT_ANTHROPIC_VERSION));
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        if let Ok(val) = HeaderValue::from_str(&self.user_agent) {
            headers.insert(reqwest::header::USER_AGENT, val);
        }

        if !self.api_key.is_empty() && let Ok(val) = HeaderValue::from_str(&self.api_key) {
            headers.insert("x-api-key", val);
        }

        if !self.beta_features.is_empty() {
            let beta_value = self.beta_features.join(",");
            if let Ok(val) = HeaderValue::from_str(&beta_value) {
                headers.insert("anthropic-beta", val);
            }
        }

        // Merge any user-provided default headers (they override built-in ones)
        for (key, value) in &self.default_headers {
            headers.insert(key, value.clone());
        }

        headers
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            default_headers: HeaderMap::new(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            beta_features: Vec::new(),
        };
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_build_headers_without_api_key() {
        let config = ClientConfig {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            default_headers: HeaderMap::new(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            beta_features: Vec::new(),
        };
        let headers = config.build_headers();
        assert_eq!(headers.get("anthropic-version").unwrap(), "2023-06-01");
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(headers.get("user-agent").unwrap(), DEFAULT_USER_AGENT);
        assert!(headers.get("x-api-key").is_none());
    }

    #[test]
    fn test_build_headers_with_api_key() {
        let config = ClientConfig {
            api_key: "sk-ant-test-key".to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            default_headers: HeaderMap::new(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            beta_features: Vec::new(),
        };
        let headers = config.build_headers();
        assert_eq!(headers.get("x-api-key").unwrap(), "sk-ant-test-key");
    }

    #[test]
    fn test_custom_default_headers_override() {
        let mut custom = HeaderMap::new();
        custom.insert("anthropic-version", HeaderValue::from_static("2024-01-01"));

        let config = ClientConfig {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            default_headers: custom,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            beta_features: Vec::new(),
        };
        let headers = config.build_headers();
        assert_eq!(headers.get("anthropic-version").unwrap(), "2024-01-01");
    }
}
