use crate::modules::billing::domain::{CorridorPolicy, JurisdictionProfile, PayoutPreference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingPermission {
    ReadPolicy,
}

pub fn is_allowed(role: &str, permission: BillingPermission) -> bool {
    if permission != BillingPermission::ReadPolicy {
        return false;
    }
    matches!(
        role,
        "platform_admin" | "platform_finance_operator" | "tenant_admin"
    )
}

pub fn list_jurisdictions() -> Vec<JurisdictionProfile> {
    vec![JurisdictionProfile {
        jurisdiction_code: "SG".to_string(),
        jurisdiction_name: "Singapore".to_string(),
        regulator_name: "MAS".to_string(),
        launch_phase: "launch_active".to_string(),
        supports_fiat_collection: true,
        supports_fiat_payout: true,
        supports_crypto_settlement: false,
        jurisdiction_status: "active".to_string(),
    }]
}

pub fn list_corridor_policies() -> Vec<CorridorPolicy> {
    vec![CorridorPolicy {
        corridor_policy_id: "corridor-sg-launch".to_string(),
        policy_name: "SG Launch Standard Corridor".to_string(),
        payer_jurisdiction_code: "SG".to_string(),
        payee_jurisdiction_code: "SG".to_string(),
        price_currency_code: "USD".to_string(),
        allowed_collection_currencies: vec!["USD".to_string(), "SGD".to_string()],
        allowed_payout_currencies: vec!["USD".to_string(), "SGD".to_string()],
        route_mode: "partner_routed".to_string(),
        requires_manual_review: false,
        allows_crypto: false,
        corridor_status: "active".to_string(),
    }]
}

pub fn list_payout_preferences(beneficiary_subject_id: &str) -> Vec<PayoutPreference> {
    vec![PayoutPreference {
        payout_preference_id: "pref-sg-default".to_string(),
        beneficiary_subject_type: "organization".to_string(),
        beneficiary_subject_id: beneficiary_subject_id.to_string(),
        destination_jurisdiction_code: "SG".to_string(),
        preferred_currency_code: "SGD".to_string(),
        payout_method: "bank_transfer".to_string(),
        preferred_provider_key: "offline_bank".to_string(),
        preferred_provider_account_id: "acct-sg-001".to_string(),
        is_default: true,
        preference_status: "active".to_string(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_allowed_roles_can_read_policy() {
        assert!(is_allowed("platform_admin", BillingPermission::ReadPolicy));
        assert!(is_allowed(
            "platform_finance_operator",
            BillingPermission::ReadPolicy
        ));
        assert!(is_allowed("tenant_admin", BillingPermission::ReadPolicy));
        assert!(!is_allowed("tenant_auditor", BillingPermission::ReadPolicy));
    }

    #[test]
    fn seed_models_match_sg_launch_baseline() {
        let jurisdictions = list_jurisdictions();
        assert_eq!(jurisdictions.len(), 1);
        assert_eq!(jurisdictions[0].jurisdiction_code, "SG");
        let corridors = list_corridor_policies();
        assert_eq!(corridors[0].policy_name, "SG Launch Standard Corridor");
    }
}
