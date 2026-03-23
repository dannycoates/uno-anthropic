pub mod types;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::multipart;

use crate::client::Client;
use crate::error::Error;
use crate::types::Page;

pub use self::types::*;

/// Service for the Files API (beta).
///
/// Access via `client.files()`.
///
/// This API requires the `files-api-2025-04-14` beta header.
pub struct FileService<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> FileService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Build the beta header for files API requests.
    fn beta_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "anthropic-beta",
            HeaderValue::from_static("files-api-2025-04-14"),
        );
        headers
    }

    /// Upload a file.
    ///
    /// Calls `POST /v1/files` with multipart form data.
    pub async fn upload(
        &self,
        file_data: Vec<u8>,
        filename: &str,
        mime_type: &str,
    ) -> Result<FileMetadata, Error> {
        let inner = &self.client.inner;
        let url = format!("{}/v1/files", inner.config.base_url.trim_end_matches('/'));
        let headers = inner.config.build_headers();
        let beta_headers = Self::beta_headers();

        let part = multipart::Part::bytes(file_data)
            .file_name(filename.to_string())
            .mime_str(mime_type)
            .map_err(|e| Error::StreamError(format!("Invalid MIME type: {}", e)))?;

        let form = multipart::Form::new().part("file", part);

        let mut request = inner.http.post(&url).headers(headers).headers(beta_headers);
        request = request.multipart(form);

        let req = request.build().map_err(Error::Http)?;
        let response = inner.http.execute(req).await.map_err(Error::Http)?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body_bytes = response.bytes().await.map_err(Error::Http)?;
            let error_body = serde_json::from_slice::<crate::error::ApiErrorResponse>(&body_bytes)
                .map(|r| r.error)
                .unwrap_or_else(|_| crate::error::ApiErrorBody {
                    error_type: "unknown_error".to_string(),
                    message: String::from_utf8_lossy(&body_bytes).to_string(),
                });
            return Err(Error::Api {
                status,
                body: error_body,
                retry_after: None,
            });
        }

        let bytes = response.bytes().await.map_err(Error::Http)?;
        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    /// Get file metadata.
    ///
    /// Calls `GET /v1/files/{file_id}`.
    pub async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata, Error> {
        let path = format!("files/{}", file_id);
        let headers = Self::beta_headers();
        self.client.get(&path, Some(&headers)).await
    }

    /// Download a file's contents.
    ///
    /// Calls `GET /v1/files/{file_id}/content`.
    pub async fn download(&self, file_id: &str) -> Result<bytes::Bytes, Error> {
        let path = format!("files/{}/content", file_id);
        let headers = Self::beta_headers();
        self.client
            .execute_raw("GET", &path, None::<&()>, Some(&headers))
            .await
    }

    /// List files.
    ///
    /// Calls `GET /v1/files` with optional pagination parameters.
    pub async fn list(&self, params: FileListParams) -> Result<Page<FileMetadata>, Error> {
        let query = params.to_query_string();
        let path = if query.is_empty() {
            "files".to_string()
        } else {
            format!("files?{}", query)
        };
        let headers = Self::beta_headers();
        self.client.get(&path, Some(&headers)).await
    }

    /// Delete a file.
    ///
    /// Calls `DELETE /v1/files/{file_id}`.
    pub async fn delete(&self, file_id: &str) -> Result<DeletedFile, Error> {
        let path = format!("files/{}", file_id);
        let headers = Self::beta_headers();
        self.client.delete(&path, Some(&headers)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_list_params_default() {
        let params = FileListParams::default();
        assert_eq!(params.to_query_string(), "");
    }
}
