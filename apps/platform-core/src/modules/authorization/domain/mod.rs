use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationExpectationSnapshot {
    pub grant_scope: String,
    pub access_mode: String,
    pub expected_valid_days: Option<i32>,
    pub export_allowed: bool,
    pub requires_poc_environment: bool,
}

impl Default for AuthorizationExpectationSnapshot {
    fn default() -> Self {
        Self {
            grant_scope: "product_level".to_string(),
            access_mode: "read_only".to_string(),
            expected_valid_days: None,
            export_allowed: false,
            requires_poc_environment: false,
        }
    }
}
