use serde::{Deserialize, Serialize};

use super::content::TextBlockParam;

/// The source of a document in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentSource {
    Base64(Base64DocumentSource),
    Text(PlainTextSource),
    Content(ContentBlockSource),
    Url(UrlDocumentSource),
}

/// A base64-encoded document source (PDF).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64DocumentSource {
    pub media_type: String,
    pub data: String,
}

/// A plain text document source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainTextSource {
    pub media_type: String,
    pub data: String,
}

/// A content block document source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockSource {
    pub content: Vec<TextBlockParam>,
}

/// A URL document source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlDocumentSource {
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_document_source_roundtrip() {
        let source = DocumentSource::Base64(Base64DocumentSource {
            media_type: "application/pdf".to_string(),
            data: "JVBERi0=".to_string(),
        });
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""type":"base64""#));
        assert!(json.contains(r#""media_type":"application/pdf""#));
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            DocumentSource::Base64(b) => {
                assert_eq!(b.media_type, "application/pdf");
                assert_eq!(b.data, "JVBERi0=");
            }
            _ => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_plain_text_source_roundtrip() {
        let source = DocumentSource::Text(PlainTextSource {
            media_type: "text/plain".to_string(),
            data: "Hello, document!".to_string(),
        });
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""type":"text""#));
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            DocumentSource::Text(t) => assert_eq!(t.data, "Hello, document!"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_url_document_source_roundtrip() {
        let source = DocumentSource::Url(UrlDocumentSource {
            url: "https://example.com/doc.pdf".to_string(),
        });
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""type":"url""#));
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            DocumentSource::Url(u) => assert_eq!(u.url, "https://example.com/doc.pdf"),
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_content_block_source_roundtrip() {
        let source = DocumentSource::Content(ContentBlockSource {
            content: vec![TextBlockParam::new("some content")],
        });
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""type":"content""#));
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            DocumentSource::Content(c) => {
                assert_eq!(c.content.len(), 1);
                assert_eq!(c.content[0].text, "some content");
            }
            _ => panic!("Expected Content variant"),
        }
    }
}
