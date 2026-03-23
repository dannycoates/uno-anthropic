pub mod types;

use reqwest::header::{HeaderMap, HeaderValue};

use crate::client::Client;
use crate::error::Error;
use crate::types::Page;

pub use self::types::*;

/// Service for the Skills API (beta).
///
/// Access via `client.skills()`.
///
/// This API requires the `skills-2025-10-02` beta header.
pub struct SkillService<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> SkillService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Build the beta header for skills API requests.
    fn beta_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "anthropic-beta",
            HeaderValue::from_static("skills-2025-10-02"),
        );
        headers
    }

    /// Create a new skill.
    ///
    /// Calls `POST /v1/skills`.
    pub async fn create(&self, params: SkillCreateParams) -> Result<Skill, Error> {
        let headers = Self::beta_headers();
        self.client.post("skills", &params, Some(&headers)).await
    }

    /// Get a skill by ID.
    ///
    /// Calls `GET /v1/skills/{skill_id}`.
    pub async fn get(&self, skill_id: &str) -> Result<Skill, Error> {
        let path = format!("skills/{}", skill_id);
        let headers = Self::beta_headers();
        self.client.get(&path, Some(&headers)).await
    }

    /// List skills.
    ///
    /// Calls `GET /v1/skills` with optional pagination parameters.
    pub async fn list(&self, params: SkillListParams) -> Result<Page<Skill>, Error> {
        let query = params.to_query_string();
        let path = if query.is_empty() {
            "skills".to_string()
        } else {
            format!("skills?{}", query)
        };
        let headers = Self::beta_headers();
        self.client.get(&path, Some(&headers)).await
    }

    /// Delete a skill.
    ///
    /// Calls `DELETE /v1/skills/{skill_id}`.
    pub async fn delete(&self, skill_id: &str) -> Result<DeletedSkill, Error> {
        let path = format!("skills/{}", skill_id);
        let headers = Self::beta_headers();
        self.client.delete(&path, Some(&headers)).await
    }

    /// Access the skill versions sub-service.
    pub fn versions(&self) -> SkillVersionService<'a> {
        SkillVersionService {
            client: self.client,
        }
    }
}

/// Service for skill version operations.
pub struct SkillVersionService<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> SkillVersionService<'a> {
    /// Build the beta header for skills API requests.
    fn beta_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "anthropic-beta",
            HeaderValue::from_static("skills-2025-10-02"),
        );
        headers
    }

    /// Create a new version for a skill.
    ///
    /// Calls `POST /v1/skills/{skill_id}/versions`.
    pub async fn create(
        &self,
        skill_id: &str,
        params: SkillVersionCreateParams,
    ) -> Result<SkillVersion, Error> {
        let path = format!("skills/{}/versions", skill_id);
        let headers = Self::beta_headers();
        self.client.post(&path, &params, Some(&headers)).await
    }

    /// Get a skill version by ID.
    ///
    /// Calls `GET /v1/skills/{skill_id}/versions/{version_id}`.
    pub async fn get(&self, skill_id: &str, version_id: &str) -> Result<SkillVersion, Error> {
        let path = format!("skills/{}/versions/{}", skill_id, version_id);
        let headers = Self::beta_headers();
        self.client.get(&path, Some(&headers)).await
    }

    /// List versions of a skill.
    ///
    /// Calls `GET /v1/skills/{skill_id}/versions` with optional pagination parameters.
    pub async fn list(
        &self,
        skill_id: &str,
        params: SkillVersionListParams,
    ) -> Result<Page<SkillVersion>, Error> {
        let query = params.to_query_string();
        let path = if query.is_empty() {
            format!("skills/{}/versions", skill_id)
        } else {
            format!("skills/{}/versions?{}", skill_id, query)
        };
        let headers = Self::beta_headers();
        self.client.get(&path, Some(&headers)).await
    }

    /// Delete a skill version.
    ///
    /// Calls `DELETE /v1/skills/{skill_id}/versions/{version_id}`.
    pub async fn delete(
        &self,
        skill_id: &str,
        version_id: &str,
    ) -> Result<DeletedSkillVersion, Error> {
        let path = format!("skills/{}/versions/{}", skill_id, version_id);
        let headers = Self::beta_headers();
        self.client.delete(&path, Some(&headers)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_list_params_default() {
        let params = SkillListParams::default();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_skill_version_list_params_default() {
        let params = SkillVersionListParams::default();
        assert_eq!(params.to_query_string(), "");
    }
}
