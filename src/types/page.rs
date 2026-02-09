use serde::Deserialize;

/// A paginated response from list endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_page() {
        let json = r#"{
            "data": [1, 2, 3],
            "has_more": true,
            "first_id": "item_001",
            "last_id": "item_003"
        }"#;
        let page: Page<u32> = serde_json::from_str(json).unwrap();
        assert_eq!(page.data, vec![1, 2, 3]);
        assert!(page.has_more);
        assert_eq!(page.first_id.as_deref(), Some("item_001"));
        assert_eq!(page.last_id.as_deref(), Some("item_003"));
    }

    #[test]
    fn test_deserialize_page_minimal() {
        let json = r#"{
            "data": [],
            "has_more": false
        }"#;
        let page: Page<String> = serde_json::from_str(json).unwrap();
        assert!(page.data.is_empty());
        assert!(!page.has_more);
        assert!(page.first_id.is_none());
        assert!(page.last_id.is_none());
    }

    #[test]
    fn test_deserialize_page_with_objects() {
        #[derive(Debug, Clone, Deserialize, PartialEq)]
        struct Item {
            id: String,
            name: String,
        }

        let json = r#"{
            "data": [
                {"id": "1", "name": "first"},
                {"id": "2", "name": "second"}
            ],
            "has_more": false,
            "first_id": "1",
            "last_id": "2"
        }"#;
        let page: Page<Item> = serde_json::from_str(json).unwrap();
        assert_eq!(page.data.len(), 2);
        assert_eq!(page.data[0].id, "1");
        assert_eq!(page.data[1].name, "second");
    }
}
