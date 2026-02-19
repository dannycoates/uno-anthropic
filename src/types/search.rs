use serde::{Deserialize, Serialize};

/// User location for web search queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    #[serde(rename = "type")]
    pub location_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

/// Backward-compatible alias for [`UserLocation`].
pub type WebSearchUserLocation = UserLocation;

impl UserLocation {
    /// Create a new approximate user location.
    pub fn approximate() -> Self {
        Self {
            location_type: "approximate".to_string(),
            city: None,
            region: None,
            country: None,
            timezone: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_user_location_serialize() {
        let loc = WebSearchUserLocation {
            location_type: "approximate".to_string(),
            city: Some("San Francisco".to_string()),
            region: Some("California".to_string()),
            country: Some("US".to_string()),
            timezone: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains(r#""type":"approximate""#));
        assert!(json.contains(r#""city":"San Francisco""#));
        assert!(!json.contains("timezone"));
    }

    #[test]
    fn test_web_search_user_location_approximate() {
        let loc = WebSearchUserLocation::approximate();
        let json = serde_json::to_string(&loc).unwrap();
        assert_eq!(json, r#"{"type":"approximate"}"#);
    }

    #[test]
    fn test_web_search_user_location_deserialize() {
        let json = r#"{"type":"approximate","city":"NYC","country":"US"}"#;
        let loc: WebSearchUserLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.location_type, "approximate");
        assert_eq!(loc.city.as_deref(), Some("NYC"));
        assert_eq!(loc.country.as_deref(), Some("US"));
        assert!(loc.region.is_none());
        assert!(loc.timezone.is_none());
    }
}
