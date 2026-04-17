#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IamPermission {
    OrgRegister,
    OrgRead,
    IdentityWrite,
    IdentityRead,
    SessionRead,
}

pub fn is_allowed(role: &str, permission: IamPermission) -> bool {
    match permission {
        IamPermission::OrgRegister => matches!(role, "platform_admin" | "tenant_admin"),
        IamPermission::OrgRead | IamPermission::IdentityRead => matches!(
            role,
            "platform_admin"
                | "tenant_admin"
                | "tenant_operator"
                | "platform_auditor"
                | "platform_finance_operator"
        ),
        IamPermission::IdentityWrite => matches!(role, "platform_admin" | "tenant_admin"),
        IamPermission::SessionRead => matches!(
            role,
            "platform_admin" | "tenant_admin" | "tenant_operator" | "developer"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_matrix_for_org_register() {
        assert!(is_allowed("platform_admin", IamPermission::OrgRegister));
        assert!(is_allowed("tenant_admin", IamPermission::OrgRegister));
        assert!(!is_allowed("tenant_operator", IamPermission::OrgRegister));
    }

    #[test]
    fn role_matrix_for_identity_write() {
        assert!(is_allowed("platform_admin", IamPermission::IdentityWrite));
        assert!(is_allowed("tenant_admin", IamPermission::IdentityWrite));
        assert!(!is_allowed("developer", IamPermission::IdentityWrite));
    }

    #[test]
    fn role_matrix_for_session_read() {
        assert!(is_allowed("developer", IamPermission::SessionRead));
        assert!(is_allowed("tenant_operator", IamPermission::SessionRead));
        assert!(!is_allowed("guest", IamPermission::SessionRead));
    }
}
