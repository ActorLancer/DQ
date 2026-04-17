use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IamPermission {
    OrgRegister,
    OrgRead,
    IdentityWrite,
    IdentityRead,
    SessionRead,
    StepUpWrite,
    StepUpRead,
    MfaRead,
    MfaWrite,
    AccessPolicyRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleDomain {
    Tenant,
    Platform,
    Audit,
    Developer,
}

#[derive(Debug, Clone)]
pub struct RoleSeed {
    pub role: &'static str,
    pub domain: RoleDomain,
    pub permissions: HashSet<IamPermission>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskAction {
    ProductFreeze,
    CompensationPayout,
    EvidenceExport,
    EvidenceReplay,
    PermissionChange,
}

pub fn role_seeds() -> &'static [RoleSeed] {
    static RBAC_SEEDS: OnceLock<Vec<RoleSeed>> = OnceLock::new();
    RBAC_SEEDS.get_or_init(|| {
        vec![
            RoleSeed {
                role: "tenant_admin",
                domain: RoleDomain::Tenant,
                permissions: HashSet::from([
                    IamPermission::OrgRegister,
                    IamPermission::OrgRead,
                    IamPermission::IdentityWrite,
                    IamPermission::IdentityRead,
                    IamPermission::SessionRead,
                    IamPermission::StepUpWrite,
                    IamPermission::StepUpRead,
                    IamPermission::MfaRead,
                    IamPermission::MfaWrite,
                    IamPermission::AccessPolicyRead,
                ]),
            },
            RoleSeed {
                role: "tenant_operator",
                domain: RoleDomain::Tenant,
                permissions: HashSet::from([
                    IamPermission::OrgRead,
                    IamPermission::IdentityRead,
                    IamPermission::SessionRead,
                    IamPermission::StepUpRead,
                    IamPermission::MfaRead,
                ]),
            },
            RoleSeed {
                role: "platform_admin",
                domain: RoleDomain::Platform,
                permissions: HashSet::from([
                    IamPermission::OrgRegister,
                    IamPermission::OrgRead,
                    IamPermission::IdentityWrite,
                    IamPermission::IdentityRead,
                    IamPermission::SessionRead,
                    IamPermission::StepUpWrite,
                    IamPermission::StepUpRead,
                    IamPermission::MfaRead,
                    IamPermission::MfaWrite,
                    IamPermission::AccessPolicyRead,
                ]),
            },
            RoleSeed {
                role: "platform_finance_operator",
                domain: RoleDomain::Platform,
                permissions: HashSet::from([IamPermission::OrgRead, IamPermission::IdentityRead]),
            },
            RoleSeed {
                role: "platform_auditor",
                domain: RoleDomain::Audit,
                permissions: HashSet::from([
                    IamPermission::OrgRead,
                    IamPermission::IdentityRead,
                    IamPermission::SessionRead,
                    IamPermission::StepUpRead,
                    IamPermission::AccessPolicyRead,
                ]),
            },
            RoleSeed {
                role: "developer",
                domain: RoleDomain::Developer,
                permissions: HashSet::from([IamPermission::SessionRead, IamPermission::MfaRead]),
            },
        ]
    })
}

pub fn is_allowed(role: &str, permission: IamPermission) -> bool {
    role_seeds()
        .iter()
        .find(|seed| seed.role == role)
        .map(|seed| seed.permissions.contains(&permission))
        .unwrap_or(false)
}

pub fn high_risk_action_requires_step_up(action: HighRiskAction) -> bool {
    match action {
        HighRiskAction::ProductFreeze
        | HighRiskAction::CompensationPayout
        | HighRiskAction::EvidenceExport
        | HighRiskAction::EvidenceReplay
        | HighRiskAction::PermissionChange => true,
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

    #[test]
    fn role_matrix_for_step_up_write() {
        assert!(is_allowed("platform_admin", IamPermission::StepUpWrite));
        assert!(is_allowed("tenant_admin", IamPermission::StepUpWrite));
        assert!(!is_allowed("platform_auditor", IamPermission::StepUpWrite));
    }

    #[test]
    fn all_high_risk_actions_require_step_up() {
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::ProductFreeze
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::CompensationPayout
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::EvidenceExport
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::EvidenceReplay
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::PermissionChange
        ));
    }
}
