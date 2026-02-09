use futures::stream::Stream;
use tokio::io::AsyncBufReadExt;
use tokio_stream::wrappers::LinesStream;
use tokio_stream::StreamExt;

use crate::error::Error;

/// A raw SSE event parsed from the byte stream.
#[derive(Debug, Clone, Default)]
pub struct RawSseEvent {
    /// The `event:` field value (e.g., "message_start", "content_block_delta").
    pub event: Option<String>,
    /// The `data:` field value. Multiple data lines are concatenated with `\n`.
    pub data: Option<String>,
    /// The `id:` field value.
    pub id: Option<String>,
    /// The `retry:` field value in milliseconds.
    pub retry: Option<u64>,
}

/// Parse an SSE byte stream into a stream of `RawSseEvent`.
///
/// Follows the SSE spec:
/// - Lines starting with `:` are comments (skipped).
/// - Empty lines dispatch the current event.
/// - `event:`, `data:`, `id:`, `retry:` fields are parsed.
/// - Multiple `data:` lines are concatenated with `\n`.
pub fn parse_sse_stream(
    response: reqwest::Response,
) -> impl Stream<Item = Result<RawSseEvent, Error>> {
    let byte_stream = response.bytes_stream();

    // Convert the byte stream into an AsyncRead, then split into lines
    let reader = tokio_util::io::StreamReader::new(
        byte_stream.map(|result| result.map_err(std::io::Error::other)),
    );
    let buf_reader = tokio::io::BufReader::new(reader);
    let lines = LinesStream::new(buf_reader.lines());

    // State machine: accumulate fields until an empty line dispatches the event
    futures::stream::unfold(
        (lines, RawSseEvent::default()),
        |(mut lines, mut current)| async move {
            loop {
                match lines.next().await {
                    Some(Ok(line)) => {
                        if line.is_empty() {
                            // Empty line = dispatch event
                            if current.event.is_some() || current.data.is_some() {
                                let event = std::mem::take(&mut current);
                                return Some((Ok(event), (lines, current)));
                            }
                            // No accumulated data, skip
                            continue;
                        }

                        // Comment line: starts with ':'
                        if line.starts_with(':') {
                            continue;
                        }

                        // Parse field: name:value
                        if let Some((field, value)) = parse_field(&line) {
                            match field {
                                "event" => {
                                    current.event = Some(value.to_string());
                                }
                                "data" => {
                                    match &mut current.data {
                                        Some(existing) => {
                                            existing.push('\n');
                                            existing.push_str(value);
                                        }
                                        None => {
                                            current.data = Some(value.to_string());
                                        }
                                    }
                                }
                                "id" => {
                                    current.id = Some(value.to_string());
                                }
                                "retry" => {
                                    if let Ok(ms) = value.trim().parse::<u64>() {
                                        current.retry = Some(ms);
                                    }
                                }
                                _ => {
                                    // Unknown field, ignore per spec
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        return Some((
                            Err(Error::StreamError(format!("SSE read error: {e}"))),
                            (lines, current),
                        ));
                    }
                    None => {
                        // Stream ended. Dispatch any remaining event.
                        if current.event.is_some() || current.data.is_some() {
                            let event = std::mem::take(&mut current);
                            return Some((Ok(event), (lines, current)));
                        }
                        return None;
                    }
                }
            }
        },
    )
}

/// Parse an SSE field line into (field_name, value).
///
/// Format: `field: value` or `field:value` (space after colon is optional but trimmed).
fn parse_field(line: &str) -> Option<(&str, &str)> {
    let colon_pos = line.find(':')?;
    let field = &line[..colon_pos];
    let mut value = &line[colon_pos + 1..];
    // Strip a single leading space after the colon, per SSE spec
    if value.starts_with(' ') {
        value = &value[1..];
    }
    Some((field, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_with_space() {
        let (field, value) = parse_field("event: message_start").unwrap();
        assert_eq!(field, "event");
        assert_eq!(value, "message_start");
    }

    #[test]
    fn test_parse_field_without_space() {
        let (field, value) = parse_field("data:{\"type\":\"ping\"}").unwrap();
        assert_eq!(field, "data");
        assert_eq!(value, "{\"type\":\"ping\"}");
    }

    #[test]
    fn test_parse_field_empty_value() {
        let (field, value) = parse_field("data:").unwrap();
        assert_eq!(field, "data");
        assert_eq!(value, "");
    }

    #[test]
    fn test_parse_field_no_colon() {
        assert!(parse_field("no colon here").is_none());
    }

    #[test]
    fn test_parse_field_colon_in_value() {
        let (field, value) = parse_field("data: {\"key\": \"value\"}").unwrap();
        assert_eq!(field, "data");
        assert_eq!(value, "{\"key\": \"value\"}");
    }

    #[tokio::test]
    async fn test_parse_sse_stream_basic() {
        let body = "event: message_start\ndata: {\"type\":\"message_start\"}\n\nevent: ping\ndata: {}\n\n";
        let response = http::Response::builder()
            .status(200)
            .body(body)
            .unwrap();
        let response = reqwest::Response::from(response);

        let events: Vec<_> = futures::StreamExt::collect::<Vec<_>>(parse_sse_stream(response))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event.as_deref(), Some("message_start"));
        assert_eq!(events[0].data.as_deref(), Some("{\"type\":\"message_start\"}"));
        assert_eq!(events[1].event.as_deref(), Some("ping"));
        assert_eq!(events[1].data.as_deref(), Some("{}"));
    }

    #[tokio::test]
    async fn test_parse_sse_stream_multiline_data() {
        let body = "event: test\ndata: line1\ndata: line2\ndata: line3\n\n";
        let response = http::Response::builder()
            .status(200)
            .body(body)
            .unwrap();
        let response = reqwest::Response::from(response);

        let events: Vec<_> = futures::StreamExt::collect::<Vec<_>>(parse_sse_stream(response))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data.as_deref(), Some("line1\nline2\nline3"));
    }

    #[tokio::test]
    async fn test_parse_sse_stream_comments_skipped() {
        let body = ": this is a comment\nevent: ping\ndata: {}\n\n";
        let response = http::Response::builder()
            .status(200)
            .body(body)
            .unwrap();
        let response = reqwest::Response::from(response);

        let events: Vec<_> = futures::StreamExt::collect::<Vec<_>>(parse_sse_stream(response))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("ping"));
    }

    #[tokio::test]
    async fn test_parse_sse_stream_empty_lines_between_events() {
        let body = "\n\nevent: a\ndata: 1\n\n\n\nevent: b\ndata: 2\n\n";
        let response = http::Response::builder()
            .status(200)
            .body(body)
            .unwrap();
        let response = reqwest::Response::from(response);

        let events: Vec<_> = futures::StreamExt::collect::<Vec<_>>(parse_sse_stream(response))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event.as_deref(), Some("a"));
        assert_eq!(events[1].event.as_deref(), Some("b"));
    }
}
