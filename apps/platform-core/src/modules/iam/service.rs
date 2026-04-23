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
    SsoRead,
    SsoWrite,
    FabricRead,
    FabricWrite,
    FabricRegistryRead,
    FabricRegistryManage,
    FabricIdentityRead,
    FabricIdentityIssue,
    FabricIdentityRevoke,
    CertificateRead,
    CertificateRevoke,
    SessionWrite,
    RoleChangeWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleDomain {
    Tenant,
    Platform,
    Audit,
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
    FabricIdentityIssue,
    FabricIdentityRevoke,
    CertificateRevoke,
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
                    IamPermission::SsoRead,
                    IamPermission::SsoWrite,
                    IamPermission::FabricRead,
                    IamPermission::FabricWrite,
                    IamPermission::FabricRegistryRead,
                    IamPermission::FabricRegistryManage,
                    IamPermission::FabricIdentityRead,
                    IamPermission::FabricIdentityIssue,
                    IamPermission::FabricIdentityRevoke,
                    IamPermission::CertificateRead,
                    IamPermission::CertificateRevoke,
                    IamPermission::SessionWrite,
                    IamPermission::RoleChangeWrite,
                ]),
            },
            RoleSeed {
                role: "tenant_developer",
                domain: RoleDomain::Tenant,
                permissions: HashSet::from([IamPermission::SessionRead, IamPermission::MfaRead]),
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
                    IamPermission::SsoRead,
                    IamPermission::SsoWrite,
                    IamPermission::FabricRead,
                    IamPermission::FabricWrite,
                    IamPermission::FabricRegistryRead,
                    IamPermission::FabricRegistryManage,
                    IamPermission::FabricIdentityRead,
                    IamPermission::FabricIdentityIssue,
                    IamPermission::FabricIdentityRevoke,
                    IamPermission::CertificateRead,
                    IamPermission::CertificateRevoke,
                    IamPermission::SessionWrite,
                    IamPermission::RoleChangeWrite,
                ]),
            },
            RoleSeed {
                role: "platform_risk_settlement",
                domain: RoleDomain::Platform,
                permissions: HashSet::from([IamPermission::OrgRead, IamPermission::IdentityRead]),
            },
            RoleSeed {
                role: "platform_audit_security",
                domain: RoleDomain::Audit,
                permissions: HashSet::from([
                    IamPermission::OrgRead,
                    IamPermission::IdentityRead,
                    IamPermission::SessionRead,
                    IamPermission::StepUpRead,
                    IamPermission::AccessPolicyRead,
                    IamPermission::SsoRead,
                    IamPermission::FabricRead,
                    IamPermission::FabricIdentityRead,
                    IamPermission::CertificateRead,
                ]),
            },
            RoleSeed {
                role: "fabric_ca_admin",
                domain: RoleDomain::Platform,
                permissions: HashSet::from([
                    IamPermission::OrgRead,
                    IamPermission::IdentityRead,
                    IamPermission::SessionRead,
                    IamPermission::StepUpRead,
                    IamPermission::StepUpWrite,
                    IamPermission::AccessPolicyRead,
                    IamPermission::FabricRead,
                    IamPermission::FabricWrite,
                    IamPermission::FabricRegistryRead,
                    IamPermission::FabricRegistryManage,
                    IamPermission::FabricIdentityRead,
                    IamPermission::FabricIdentityIssue,
                    IamPermission::FabricIdentityRevoke,
                    IamPermission::CertificateRead,
                    IamPermission::CertificateRevoke,
                ]),
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
        | HighRiskAction::PermissionChange
        | HighRiskAction::FabricIdentityIssue
        | HighRiskAction::FabricIdentityRevoke
        | HighRiskAction::CertificateRevoke => true,
    }
}

#[cfg(test)]
mod tests {}
