use crate::modules::billing::domain::{CorridorPolicy, JurisdictionProfile, PayoutPreference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingPermission {
    JurisdictionRead,
    JurisdictionManage,
    CorridorRead,
    CorridorManage,
    PayoutPreferenceRead,
    PayoutPreferenceManage,
    PaymentIntentRead,
    PaymentIntentCreate,
    PaymentIntentCancel,
    PaymentIntentProcessResult,
    OrderLock,
    BillingEventRead,
    PayoutRead,
    ReconciliationRead,
    ReconciliationImport,
    PayoutExecuteManual,
    RefundExecute,
    CompensationExecute,
    DisputeCaseCreate,
    DisputeEvidenceUpload,
    DisputeCaseResolve,
    MockPaymentSimulate,
}

pub fn is_allowed(role: &str, permission: BillingPermission) -> bool {
    match permission {
        BillingPermission::JurisdictionRead | BillingPermission::CorridorRead => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
        ),
        BillingPermission::JurisdictionManage | BillingPermission::CorridorManage => {
            matches!(role, "platform_admin" | "platform_finance_operator")
        }
        BillingPermission::PayoutPreferenceRead => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
        ),
        BillingPermission::PayoutPreferenceManage => matches!(
            role,
            "platform_admin" | "platform_finance_operator" | "tenant_admin"
        ),
        BillingPermission::PaymentIntentRead => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
                | "tenant_operator"
        ),
        BillingPermission::BillingEventRead => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
                | "tenant_operator"
        ),
        BillingPermission::PayoutRead => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
                | "tenant_operator"
        ),
        BillingPermission::ReconciliationRead => matches!(
            role,
            "platform_admin" | "platform_finance_operator" | "platform_risk_settlement"
        ),
        BillingPermission::RefundExecute
        | BillingPermission::CompensationExecute
        | BillingPermission::ReconciliationImport
        | BillingPermission::PayoutExecuteManual => matches!(
            role,
            "platform_admin" | "platform_finance_operator" | "platform_risk_settlement"
        ),
        BillingPermission::DisputeCaseCreate | BillingPermission::DisputeEvidenceUpload => {
            matches!(role, "buyer_operator")
        }
        BillingPermission::DisputeCaseResolve => matches!(role, "platform_risk_settlement"),
        BillingPermission::PaymentIntentCreate | BillingPermission::PaymentIntentCancel => {
            matches!(
                role,
                "platform_admin"
                    | "platform_finance_operator"
                    | "platform_risk_settlement"
                    | "tenant_admin"
            )
        }
        BillingPermission::PaymentIntentProcessResult => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
        ),
        BillingPermission::OrderLock => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
        ),
        BillingPermission::MockPaymentSimulate => matches!(
            role,
            "platform_admin"
                | "platform_finance_operator"
                | "platform_risk_settlement"
                | "tenant_admin"
        ),
    }
}

pub fn list_jurisdictions() -> Vec<JurisdictionProfile> {
    vec![JurisdictionProfile {
        jurisdiction_code: "SG".to_string(),
        jurisdiction_name: "Singapore".to_string(),
        regulator_name: Some("MAS".to_string()),
        launch_phase: "launch_active".to_string(),
        supports_fiat_collection: true,
        supports_fiat_payout: true,
        supports_crypto_settlement: false,
        jurisdiction_status: "active".to_string(),
        policy_snapshot: serde_json::json!({
            "launch_scope": "initial_production",
            "price_currency": "USD"
        }),
    }]
}

pub fn list_corridor_policies() -> Vec<CorridorPolicy> {
    vec![CorridorPolicy {
        corridor_policy_id: "corridor-sg-launch".to_string(),
        policy_name: "SG Launch Standard Corridor".to_string(),
        payer_jurisdiction_code: "SG".to_string(),
        payee_jurisdiction_code: "SG".to_string(),
        product_scope: "general".to_string(),
        price_currency_code: "USD".to_string(),
        allowed_collection_currencies: vec!["USD".to_string(), "SGD".to_string()],
        allowed_payout_currencies: vec!["USD".to_string(), "SGD".to_string()],
        route_mode: "partner_routed".to_string(),
        requires_manual_review: false,
        allows_crypto: false,
        corridor_status: "active".to_string(),
        effective_from: Some("2026-04-08T00:00:00.000Z".to_string()),
        effective_to: None,
        policy_snapshot: serde_json::json!({"real_payment_enabled": true}),
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
        preferred_provider_account_id: None,
        beneficiary_snapshot: serde_json::json!({}),
        is_default: true,
        preference_status: "active".to_string(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_expected_roles_can_read_payment_controls() {
        assert!(is_allowed(
            "platform_admin",
            BillingPermission::JurisdictionRead
        ));
        assert!(is_allowed(
            "platform_finance_operator",
            BillingPermission::CorridorRead
        ));
        assert!(is_allowed(
            "tenant_admin",
            BillingPermission::PayoutPreferenceRead
        ));
        assert!(!is_allowed(
            "tenant_operator",
            BillingPermission::JurisdictionRead
        ));
    }

    #[test]
    fn only_platform_roles_can_manage_payment_controls() {
        assert!(is_allowed(
            "platform_admin",
            BillingPermission::JurisdictionManage
        ));
        assert!(is_allowed(
            "platform_finance_operator",
            BillingPermission::CorridorManage
        ));
        assert!(!is_allowed(
            "tenant_admin",
            BillingPermission::CorridorManage
        ));
    }

    #[test]
    fn payout_preference_manage_is_tenant_or_platform_scoped() {
        assert!(is_allowed(
            "tenant_admin",
            BillingPermission::PayoutPreferenceManage
        ));
        assert!(is_allowed(
            "platform_admin",
            BillingPermission::PayoutPreferenceManage
        ));
        assert!(!is_allowed(
            "tenant_operator",
            BillingPermission::PayoutPreferenceManage
        ));
    }

    #[test]
    fn only_expected_roles_can_write_payment_intents() {
        assert!(is_allowed(
            "platform_admin",
            BillingPermission::PaymentIntentCreate
        ));
        assert!(is_allowed(
            "platform_risk_settlement",
            BillingPermission::PaymentIntentCancel
        ));
        assert!(!is_allowed(
            "tenant_operator",
            BillingPermission::PaymentIntentCancel
        ));
        assert!(is_allowed(
            "tenant_operator",
            BillingPermission::BillingEventRead
        ));
        assert!(is_allowed(
            "platform_risk_settlement",
            BillingPermission::RefundExecute
        ));
        assert!(is_allowed(
            "platform_finance_operator",
            BillingPermission::CompensationExecute
        ));
        assert!(is_allowed(
            "platform_admin",
            BillingPermission::PayoutExecuteManual
        ));
        assert!(!is_allowed(
            "tenant_admin",
            BillingPermission::RefundExecute
        ));
        assert!(!is_allowed(
            "tenant_operator",
            BillingPermission::RefundExecute
        ));
        assert!(!is_allowed(
            "tenant_admin",
            BillingPermission::PayoutExecuteManual
        ));
    }

    #[test]
    fn only_expected_roles_can_lock_order() {
        assert!(is_allowed("tenant_admin", BillingPermission::OrderLock));
        assert!(is_allowed(
            "platform_risk_settlement",
            BillingPermission::OrderLock
        ));
        assert!(!is_allowed("tenant_operator", BillingPermission::OrderLock));
    }
}
