use serde::{Deserialize, Serialize};

/// Metadata about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: String,
    #[serde(default)]
    pub downloadable: bool,
}

/// Response when a file is deleted.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedFile {
    pub id: String,
    #[serde(rename = "type")]
    pub file_type: String,
}

/// Parameters for listing files.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FileListParams {
    /// Maximum number of items to return per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor for pagination: return results after this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
    /// Cursor for pagination: return results before this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
}

impl FileListParams {
    pub(crate) fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(limit) = self.limit {
            parts.push(format!("limit={}", limit));
        }
        if let Some(ref after_id) = self.after_id {
            parts.push(format!("after_id={}", after_id));
        }
        if let Some(ref before_id) = self.before_id {
            parts.push(format!("before_id={}", before_id));
        }
        parts.join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_file_metadata() {
        let json = r#"{
            "id": "file_abc123",
            "type": "file",
            "filename": "test.pdf",
            "mime_type": "application/pdf",
            "size_bytes": 12345,
            "created_at": "2026-01-01T00:00:00Z",
            "downloadable": true
        }"#;
        let meta: FileMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.id, "file_abc123");
        assert_eq!(meta.file_type, "file");
        assert_eq!(meta.filename, "test.pdf");
        assert_eq!(meta.mime_type, "application/pdf");
        assert_eq!(meta.size_bytes, 12345);
        assert!(meta.downloadable);
    }

    #[test]
    fn test_deserialize_deleted_file() {
        let json = r#"{"id": "file_abc123", "type": "file_deleted"}"#;
        let deleted: DeletedFile = serde_json::from_str(json).unwrap();
        assert_eq!(deleted.id, "file_abc123");
        assert_eq!(deleted.file_type, "file_deleted");
    }

    #[test]
    fn test_file_list_params_empty() {
        let params = FileListParams::default();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_file_list_params_with_limit() {
        let params = FileListParams {
            limit: Some(10),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "limit=10");
    }

    #[test]
    fn test_file_list_params_with_all() {
        let params = FileListParams {
            limit: Some(5),
            after_id: Some("file_abc".to_string()),
            before_id: Some("file_xyz".to_string()),
        };
        let qs = params.to_query_string();
        assert!(qs.contains("limit=5"));
        assert!(qs.contains("after_id=file_abc"));
        assert!(qs.contains("before_id=file_xyz"));
    }
}
