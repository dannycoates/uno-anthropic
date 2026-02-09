use serde::{Deserialize, Serialize};

/// The reason a message stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    Refusal,
}

/// The role of a message participant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_reason_serialize() {
        assert_eq!(
            serde_json::to_string(&StopReason::EndTurn).unwrap(),
            r#""end_turn""#
        );
        assert_eq!(
            serde_json::to_string(&StopReason::MaxTokens).unwrap(),
            r#""max_tokens""#
        );
        assert_eq!(
            serde_json::to_string(&StopReason::StopSequence).unwrap(),
            r#""stop_sequence""#
        );
        assert_eq!(
            serde_json::to_string(&StopReason::ToolUse).unwrap(),
            r#""tool_use""#
        );
        assert_eq!(
            serde_json::to_string(&StopReason::Refusal).unwrap(),
            r#""refusal""#
        );
    }

    #[test]
    fn test_stop_reason_deserialize() {
        let reason: StopReason = serde_json::from_str(r#""end_turn""#).unwrap();
        assert_eq!(reason, StopReason::EndTurn);

        let reason: StopReason = serde_json::from_str(r#""tool_use""#).unwrap();
        assert_eq!(reason, StopReason::ToolUse);
    }

    #[test]
    fn test_role_roundtrip() {
        let user_json = serde_json::to_string(&Role::User).unwrap();
        assert_eq!(user_json, r#""user""#);
        let user: Role = serde_json::from_str(&user_json).unwrap();
        assert_eq!(user, Role::User);

        let assistant_json = serde_json::to_string(&Role::Assistant).unwrap();
        assert_eq!(assistant_json, r#""assistant""#);
        let assistant: Role = serde_json::from_str(&assistant_json).unwrap();
        assert_eq!(assistant, Role::Assistant);
    }
}
