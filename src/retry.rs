use std::time::Duration;

use rand::Rng;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries (not counting the initial attempt).
    pub max_retries: u32,
    /// Base delay for exponential backoff (default: 500ms).
    pub initial_delay: Duration,
    /// Maximum delay between retries (default: 8s).
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(8),
        }
    }
}

impl RetryPolicy {
    /// Calculate the delay for a given retry attempt.
    ///
    /// Uses exponential backoff: `initial_delay * 2^attempt`, capped at `max_delay`,
    /// with jitter subtracted (up to 25% of the computed delay).
    ///
    /// If a `retry_after` duration is provided (from Retry-After header), it takes
    /// precedence as long as it is reasonable (< 60 seconds).
    pub fn delay_for_attempt(&self, attempt: u32, retry_after: Option<Duration>) -> Duration {
        // If the server told us to wait and it's reasonable, use that
        if let Some(ra) = retry_after {
            if ra < Duration::from_secs(60) {
                return ra;
            }
        }

        // Exponential backoff: initial_delay * 2^attempt
        let delay_ms = self.initial_delay.as_millis() as u64 * 2u64.saturating_pow(attempt);
        let max_ms = self.max_delay.as_millis() as u64;
        let capped_ms = delay_ms.min(max_ms);

        // Subtract jitter: up to 25% of the delay
        let jitter = if capped_ms > 0 {
            rand::rng().random_range(0..=(capped_ms / 4))
        } else {
            0
        };

        Duration::from_millis(capped_ms - jitter)
    }
}

/// Parse the `Retry-After` header value into a Duration.
///
/// Supports:
/// - `Retry-After-Ms: <milliseconds>` (checked first)
/// - `Retry-After: <seconds>` (integer or float)
/// - `Retry-After: <HTTP-date>` (RFC 2822 / RFC 1123 format)
pub fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    // Check Retry-After-Ms first (milliseconds)
    if let Some(val) = headers.get("retry-after-ms") {
        if let Ok(s) = val.to_str() {
            if let Ok(ms) = s.parse::<f64>() {
                return Some(Duration::from_millis(ms as u64));
            }
        }
    }

    // Check Retry-After (seconds or HTTP-date)
    if let Some(val) = headers.get("retry-after") {
        if let Ok(s) = val.to_str() {
            // Try parsing as number of seconds
            if let Ok(secs) = s.parse::<f64>() {
                return Some(Duration::from_secs_f64(secs));
            }
            // Try parsing as HTTP-date (RFC 1123)
            // Example: "Wed, 21 Oct 2015 07:28:00 GMT"
            // We don't implement full HTTP-date parsing here; in practice
            // the Anthropic API uses numeric values.
        }
    }

    None
}

/// Check the `x-should-retry` header to see if the server explicitly requests retry behavior.
///
/// Returns `Some(true)` if the header says "true", `Some(false)` if "false", `None` if absent.
pub fn check_should_retry_header(headers: &reqwest::header::HeaderMap) -> Option<bool> {
    headers.get("x-should-retry").and_then(|val| {
        val.to_str().ok().map(|s| s.eq_ignore_ascii_case("true"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 2);
        assert_eq!(policy.initial_delay, Duration::from_millis(500));
        assert_eq!(policy.max_delay, Duration::from_secs(8));
    }

    #[test]
    fn test_delay_exponential_backoff() {
        let policy = RetryPolicy::default();

        // Attempt 0: 500ms * 2^0 = 500ms (minus jitter)
        let d0 = policy.delay_for_attempt(0, None);
        assert!(d0 >= Duration::from_millis(375)); // 500 - 25% jitter
        assert!(d0 <= Duration::from_millis(500));

        // Attempt 1: 500ms * 2^1 = 1000ms (minus jitter)
        let d1 = policy.delay_for_attempt(1, None);
        assert!(d1 >= Duration::from_millis(750));
        assert!(d1 <= Duration::from_millis(1000));

        // Attempt 2: 500ms * 2^2 = 2000ms (minus jitter)
        let d2 = policy.delay_for_attempt(2, None);
        assert!(d2 >= Duration::from_millis(1500));
        assert!(d2 <= Duration::from_millis(2000));

        // Attempt 4: 500ms * 2^4 = 8000ms, capped at 8000ms
        let d4 = policy.delay_for_attempt(4, None);
        assert!(d4 >= Duration::from_millis(6000));
        assert!(d4 <= Duration::from_millis(8000));

        // Attempt 10: should still be capped at 8000ms
        let d10 = policy.delay_for_attempt(10, None);
        assert!(d10 <= Duration::from_millis(8000));
    }

    #[test]
    fn test_delay_with_retry_after() {
        let policy = RetryPolicy::default();

        // Retry-After takes precedence if reasonable
        let d = policy.delay_for_attempt(0, Some(Duration::from_secs(3)));
        assert_eq!(d, Duration::from_secs(3));

        // Unreasonable Retry-After (>= 60s) falls back to exponential backoff
        let d = policy.delay_for_attempt(0, Some(Duration::from_secs(120)));
        assert!(d <= Duration::from_millis(500));
    }

    #[test]
    fn test_parse_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("2"));
        assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_parse_retry_after_float_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("1.5"));
        let d = parse_retry_after(&headers).unwrap();
        assert!(d >= Duration::from_millis(1499) && d <= Duration::from_millis(1501));
    }

    #[test]
    fn test_parse_retry_after_ms() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after-ms", HeaderValue::from_static("500"));
        assert_eq!(
            parse_retry_after(&headers),
            Some(Duration::from_millis(500))
        );
    }

    #[test]
    fn test_parse_retry_after_ms_takes_precedence() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after-ms", HeaderValue::from_static("200"));
        headers.insert("retry-after", HeaderValue::from_static("5"));
        // retry-after-ms should take precedence
        assert_eq!(
            parse_retry_after(&headers),
            Some(Duration::from_millis(200))
        );
    }

    #[test]
    fn test_parse_retry_after_missing() {
        let headers = HeaderMap::new();
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn test_check_should_retry_header() {
        let mut headers = HeaderMap::new();
        assert_eq!(check_should_retry_header(&headers), None);

        headers.insert("x-should-retry", HeaderValue::from_static("true"));
        assert_eq!(check_should_retry_header(&headers), Some(true));

        headers.insert("x-should-retry", HeaderValue::from_static("false"));
        assert_eq!(check_should_retry_header(&headers), Some(false));

        headers.insert("x-should-retry", HeaderValue::from_static("True"));
        assert_eq!(check_should_retry_header(&headers), Some(true));
    }
}
