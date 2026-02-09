use serde::{Deserialize, Serialize};

/// Configuration for enabling citations on a content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// A citation within a text response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextCitation {
    CharLocation(CharLocationCitation),
    PageLocation(PageLocationCitation),
    ContentBlockLocation(ContentBlockLocationCitation),
    WebSearchResultLocation(WebSearchResultLocationCitation),
    SearchResultLocation(SearchResultLocationCitation),
}

/// A citation referencing a character range in a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharLocationCitation {
    pub cited_text: String,
    pub document_index: u32,
    pub document_title: Option<String>,
    pub start_char_index: u32,
    pub end_char_index: u32,
}

/// A citation referencing a page range in a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLocationCitation {
    pub cited_text: String,
    pub document_index: u32,
    pub document_title: Option<String>,
    pub start_page_number: u32,
    pub end_page_number: u32,
}

/// A citation referencing content block indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockLocationCitation {
    pub cited_text: String,
    pub document_index: u32,
    pub document_title: Option<String>,
    pub start_block_index: u32,
    pub end_block_index: u32,
}

/// A citation referencing a web search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResultLocationCitation {
    pub cited_text: String,
    pub encrypted_index: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// A citation referencing a search result block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultLocationCitation {
    pub cited_text: String,
    pub document_index: u32,
    pub document_title: Option<String>,
    pub start_char_index: u32,
    pub end_char_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_location_citation_roundtrip() {
        let json = r#"{"type":"char_location","cited_text":"hello","document_index":0,"document_title":"doc.txt","start_char_index":10,"end_char_index":15}"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match &citation {
            TextCitation::CharLocation(c) => {
                assert_eq!(c.cited_text, "hello");
                assert_eq!(c.document_index, 0);
                assert_eq!(c.start_char_index, 10);
                assert_eq!(c.end_char_index, 15);
            }
            _ => panic!("Expected CharLocation variant"),
        }
        let roundtrip = serde_json::to_string(&citation).unwrap();
        let _: TextCitation = serde_json::from_str(&roundtrip).unwrap();
    }

    #[test]
    fn test_page_location_citation() {
        let json = r#"{"type":"page_location","cited_text":"text","document_index":1,"document_title":null,"start_page_number":3,"end_page_number":5}"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::PageLocation(p) => {
                assert_eq!(p.start_page_number, 3);
                assert_eq!(p.end_page_number, 5);
            }
            _ => panic!("Expected PageLocation variant"),
        }
    }

    #[test]
    fn test_web_search_result_location_citation() {
        let json = r#"{"type":"web_search_result_location","cited_text":"found text","encrypted_index":"enc123","title":"Example","url":"https://example.com"}"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::WebSearchResultLocation(w) => {
                assert_eq!(w.cited_text, "found text");
                assert_eq!(w.encrypted_index, "enc123");
                assert_eq!(w.url.as_deref(), Some("https://example.com"));
            }
            _ => panic!("Expected WebSearchResultLocation variant"),
        }
    }

    #[test]
    fn test_content_block_location_citation() {
        let json = r#"{"type":"content_block_location","cited_text":"block text","document_index":2,"document_title":"doc","start_block_index":0,"end_block_index":3}"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::ContentBlockLocation(c) => {
                assert_eq!(c.start_block_index, 0);
                assert_eq!(c.end_block_index, 3);
            }
            _ => panic!("Expected ContentBlockLocation variant"),
        }
    }

    #[test]
    fn test_search_result_location_citation() {
        let json = r#"{"type":"search_result_location","cited_text":"search text","document_index":0,"document_title":"result","start_char_index":5,"end_char_index":20}"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::SearchResultLocation(s) => {
                assert_eq!(s.cited_text, "search text");
                assert_eq!(s.start_char_index, 5);
            }
            _ => panic!("Expected SearchResultLocation variant"),
        }
    }

    #[test]
    fn test_citations_config() {
        let config = CitationsConfig {
            enabled: Some(true),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"enabled":true}"#);
    }
}
