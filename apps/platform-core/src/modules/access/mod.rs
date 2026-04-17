pub const MODULE: &str = "access";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessRule {
    pub permission_code: &'static str,
    pub scopes: &'static [&'static str],
    pub api_patterns: &'static [&'static str],
    pub button_keys: &'static [&'static str],
}

pub const ACCESS_RULES: &[AccessRule] = &[
    AccessRule {
        permission_code: "iam.identity.write",
        scopes: &["tenant", "platform"],
        api_patterns: &["/api/v1/iam/", "/api/v1/apps/", "/api/v1/users/invite"],
        button_keys: &["user.invite", "session.revoke", "device.revoke"],
    },
    AccessRule {
        permission_code: "iam.stepup.write",
        scopes: &["tenant", "platform"],
        api_patterns: &["/api/v1/iam/step-up/"],
        button_keys: &["risk.freeze", "evidence.export", "permission.change"],
    },
];

pub fn find_rule(permission_code: &str) -> Option<&'static AccessRule> {
    ACCESS_RULES
        .iter()
        .find(|rule| rule.permission_code == permission_code)
}
