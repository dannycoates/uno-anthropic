use serde::{Deserialize, Serialize};

/// The source of an image in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64(Base64ImageSource),
    Url(UrlImageSource),
}

/// A base64-encoded image source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64ImageSource {
    pub media_type: MediaType,
    pub data: String,
}

/// A URL image source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlImageSource {
    pub url: String,
}

/// Supported image media types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MediaType {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_image_source_roundtrip() {
        let source = ImageSource::Base64(Base64ImageSource {
            media_type: MediaType::Png,
            data: "iVBORw0KGgo=".to_string(),
        });
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""type":"base64""#));
        assert!(json.contains(r#""media_type":"image/png""#));
        let deserialized: ImageSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            ImageSource::Base64(b) => {
                assert_eq!(b.media_type, MediaType::Png);
                assert_eq!(b.data, "iVBORw0KGgo=");
            }
            _ => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_url_image_source_roundtrip() {
        let source = ImageSource::Url(UrlImageSource {
            url: "https://example.com/image.png".to_string(),
        });
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""type":"url""#));
        let deserialized: ImageSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            ImageSource::Url(u) => assert_eq!(u.url, "https://example.com/image.png"),
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_media_type_serialize() {
        assert_eq!(
            serde_json::to_string(&MediaType::Jpeg).unwrap(),
            r#""image/jpeg""#
        );
        assert_eq!(
            serde_json::to_string(&MediaType::Png).unwrap(),
            r#""image/png""#
        );
        assert_eq!(
            serde_json::to_string(&MediaType::Gif).unwrap(),
            r#""image/gif""#
        );
        assert_eq!(
            serde_json::to_string(&MediaType::Webp).unwrap(),
            r#""image/webp""#
        );
    }
}
