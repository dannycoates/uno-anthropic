use serde::{Deserialize, Serialize};

/// A skill resource.
#[derive(Debug, Clone, Deserialize)]
pub struct Skill {
    pub id: String,
    #[serde(rename = "type")]
    pub skill_type: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub display_title: Option<String>,
    pub source: String,
    pub latest_version: String,
}

/// A skill version resource.
#[derive(Debug, Clone, Deserialize)]
pub struct SkillVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub created_at: String,
    pub skill_id: String,
}

/// Response when a skill is deleted.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedSkill {
    pub id: String,
    #[serde(rename = "type")]
    pub skill_type: String,
}

/// Response when a skill version is deleted.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedSkillVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
}

/// Parameters for creating a skill.
#[derive(Debug, Clone, Serialize)]
pub struct SkillCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Initial version content, if creating with a version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Parameters for listing skills.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SkillListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
}

impl SkillListParams {
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

/// Parameters for creating a skill version.
#[derive(Debug, Clone, Serialize)]
pub struct SkillVersionCreateParams {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Parameters for listing skill versions.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SkillVersionListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
}

impl SkillVersionListParams {
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
    fn test_deserialize_skill() {
        let json = r#"{
            "id": "skill_abc123",
            "type": "skill",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z",
            "display_title": "My Skill",
            "source": "custom",
            "latest_version": "skver_v1"
        }"#;
        let skill: Skill = serde_json::from_str(json).unwrap();
        assert_eq!(skill.id, "skill_abc123");
        assert_eq!(skill.skill_type, "skill");
        assert_eq!(skill.display_title.as_deref(), Some("My Skill"));
        assert_eq!(skill.source, "custom");
        assert_eq!(skill.latest_version, "skver_v1");
    }

    #[test]
    fn test_deserialize_skill_version() {
        let json = r#"{
            "id": "skver_v1",
            "type": "skill_version",
            "created_at": "2026-01-01T00:00:00Z",
            "skill_id": "skill_abc123"
        }"#;
        let version: SkillVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version.id, "skver_v1");
        assert_eq!(version.version_type, "skill_version");
        assert_eq!(version.skill_id, "skill_abc123");
    }

    #[test]
    fn test_deserialize_deleted_skill() {
        let json = r#"{"id": "skill_abc123", "type": "skill_deleted"}"#;
        let deleted: DeletedSkill = serde_json::from_str(json).unwrap();
        assert_eq!(deleted.id, "skill_abc123");
        assert_eq!(deleted.skill_type, "skill_deleted");
    }

    #[test]
    fn test_skill_list_params_empty() {
        let params = SkillListParams::default();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_skill_list_params_with_limit() {
        let params = SkillListParams {
            limit: Some(10),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "limit=10");
    }

    #[test]
    fn test_skill_version_list_params_empty() {
        let params = SkillVersionListParams::default();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_skill_create_params_serialize() {
        let params = SkillCreateParams {
            display_title: Some("Test Skill".to_string()),
            description: Some("A test skill".to_string()),
            content: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""display_title":"Test Skill""#));
        assert!(json.contains(r#""description":"A test skill""#));
        assert!(!json.contains("content"));
    }

    #[test]
    fn test_skill_version_create_params_serialize() {
        let params = SkillVersionCreateParams {
            content: "skill content here".to_string(),
            description: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""content":"skill content here""#));
        assert!(!json.contains("description"));
    }
}
