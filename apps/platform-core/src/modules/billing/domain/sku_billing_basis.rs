use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SkuBillingBasisView {
    pub sku_type: String,
    pub default_event_type: Option<String>,
    pub cycle_event_type: Option<String>,
    pub usage_event_type: Option<String>,
    pub payment_trigger: String,
    pub delivery_trigger: String,
    pub acceptance_trigger: String,
    pub billing_trigger: String,
    pub settlement_cycle: String,
    pub periodic_settlement_cycle: Option<String>,
    pub refund_entry: String,
    pub refund_placeholder_entry: Option<String>,
    pub refund_placeholder_event_type: Option<String>,
    pub refund_mode: Option<String>,
    pub refund_template_code: Option<String>,
    pub compensation_entry: String,
    pub dispute_freeze_trigger: String,
    pub resume_settlement_trigger: String,
    pub policy_stage: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkuBillingBasisRule {
    pub sku_type: &'static str,
    pub default_event_type: Option<&'static str>,
    pub cycle_event_type: Option<&'static str>,
    pub usage_event_type: Option<&'static str>,
    pub payment_trigger: &'static str,
    pub delivery_trigger: &'static str,
    pub acceptance_trigger: &'static str,
    pub billing_trigger: &'static str,
    pub settlement_cycle: &'static str,
    pub refund_entry: &'static str,
    pub periodic_settlement_cycle: Option<&'static str>,
    pub refund_placeholder_entry: Option<&'static str>,
    pub refund_placeholder_event_type: Option<&'static str>,
    pub compensation_entry: &'static str,
    pub dispute_freeze_trigger: &'static str,
    pub resume_settlement_trigger: &'static str,
}

pub const FILE_STD_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "FILE_STD",
    default_event_type: Some("one_time_charge"),
    cycle_event_type: None,
    usage_event_type: None,
    payment_trigger: "order_contract_effective_lock_once",
    delivery_trigger: "seller_publish_single_package",
    acceptance_trigger: "buyer_manual_accept_or_timeout",
    billing_trigger: "bill_once_after_acceptance",
    settlement_cycle: "t_plus_1_once",
    refund_entry: "pre_acceptance_cancel_or_acceptance_failed",
    periodic_settlement_cycle: None,
    refund_placeholder_entry: None,
    refund_placeholder_event_type: None,
    compensation_entry: "delivery_defect_or_delay",
    dispute_freeze_trigger: "freeze_on_dispute_opened",
    resume_settlement_trigger: "resume_on_dispute_closed_with_ruling",
};

pub const FILE_SUB_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "FILE_SUB",
    default_event_type: Some("recurring_charge"),
    cycle_event_type: None,
    usage_event_type: None,
    payment_trigger: "lock_before_each_subscription_cycle",
    delivery_trigger: "generate_delivery_batch_each_cycle",
    acceptance_trigger: "cycle_window_manual_acceptance",
    billing_trigger: "bill_per_cycle_after_acceptance",
    settlement_cycle: "monthly_cycle",
    refund_entry: "refund_current_cycle_if_not_delivered",
    periodic_settlement_cycle: None,
    refund_placeholder_entry: None,
    refund_placeholder_event_type: None,
    compensation_entry: "compensate_on_repeated_missing_delivery",
    dispute_freeze_trigger: "freeze_future_cycles_on_dispute_opened",
    resume_settlement_trigger: "resume_after_dispute_closed_for_cycle",
};

pub const SHARE_RO_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "SHARE_RO",
    default_event_type: Some("one_time_charge"),
    cycle_event_type: Some("recurring_charge"),
    usage_event_type: None,
    payment_trigger: "lock_before_share_grant_activation",
    delivery_trigger: "readonly_share_grant_enabled",
    acceptance_trigger: "accessibility_check_passed",
    billing_trigger: "bill_once_on_grant_effective",
    settlement_cycle: "t_plus_1_once",
    refund_entry: "refund_if_grant_not_effective",
    periodic_settlement_cycle: Some("monthly_cycle"),
    refund_placeholder_entry: Some("placeholder_on_share_revoke_or_scope_cutoff"),
    refund_placeholder_event_type: Some("refund_adjustment"),
    compensation_entry: "compensate_on_scope_or_access_violation",
    dispute_freeze_trigger: "freeze_on_share_dispute_opened",
    resume_settlement_trigger: "resume_on_dispute_closed_after_fix",
};

pub const API_SUB_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "API_SUB",
    default_event_type: Some("recurring_charge"),
    cycle_event_type: None,
    usage_event_type: Some("usage_charge"),
    payment_trigger: "lock_before_subscription_cycle_start",
    delivery_trigger: "api_key_and_quota_provisioned",
    acceptance_trigger: "first_success_call_or_cycle_acceptance",
    billing_trigger: "bill_cycle_after_enable_and_acceptance",
    settlement_cycle: "monthly_cycle",
    refund_entry: "refund_current_cycle_if_unavailable",
    periodic_settlement_cycle: None,
    refund_placeholder_entry: None,
    refund_placeholder_event_type: None,
    compensation_entry: "compensate_on_sla_breach",
    dispute_freeze_trigger: "freeze_current_cycle_on_sla_dispute",
    resume_settlement_trigger: "resume_on_sla_dispute_closed",
};

pub const API_PPU_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "API_PPU",
    default_event_type: None,
    cycle_event_type: None,
    usage_event_type: Some("usage_charge"),
    payment_trigger: "lock_prepaid_quota_or_minimum_commit",
    delivery_trigger: "api_key_enabled_for_metering",
    acceptance_trigger: "usage_reconciliation_window_confirmed",
    billing_trigger: "bill_by_metered_usage",
    settlement_cycle: "daily_with_monthly_statement",
    refund_entry: "refund_failed_batch_or_unused_quota",
    periodic_settlement_cycle: None,
    refund_placeholder_entry: None,
    refund_placeholder_event_type: None,
    compensation_entry: "compensate_on_metering_or_throttling_fault",
    dispute_freeze_trigger: "freeze_metered_settlement_on_dispute",
    resume_settlement_trigger: "resume_after_metering_reconcile",
};

pub const QRY_LITE_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "QRY_LITE",
    default_event_type: Some("one_time_charge"),
    cycle_event_type: None,
    usage_event_type: None,
    payment_trigger: "lock_before_query_job_execution",
    delivery_trigger: "query_job_succeeded_result_available",
    acceptance_trigger: "result_integrity_and_download_check",
    billing_trigger: "bill_once_after_task_acceptance",
    settlement_cycle: "t_plus_1_once",
    refund_entry: "refund_if_task_failed_or_unavailable",
    periodic_settlement_cycle: None,
    refund_placeholder_entry: None,
    refund_placeholder_event_type: None,
    compensation_entry: "compensate_on_execution_unavailability",
    dispute_freeze_trigger: "freeze_on_query_result_dispute",
    resume_settlement_trigger: "resume_after_result_recheck_closed",
};

pub const SBX_STD_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "SBX_STD",
    default_event_type: Some("recurring_charge"),
    cycle_event_type: None,
    usage_event_type: None,
    payment_trigger: "lock_before_workspace_provision",
    delivery_trigger: "workspace_account_quota_ready",
    acceptance_trigger: "login_and_probe_check_passed",
    billing_trigger: "bill_after_workspace_activation_acceptance",
    settlement_cycle: "monthly_resource_cycle",
    refund_entry: "refund_if_workspace_not_ready",
    periodic_settlement_cycle: None,
    refund_placeholder_entry: None,
    refund_placeholder_event_type: None,
    compensation_entry: "compensate_on_resource_or_isolation_fault",
    dispute_freeze_trigger: "freeze_on_security_or_isolation_dispute",
    resume_settlement_trigger: "resume_after_risk_cleared_dispute_closed",
};

pub const RPT_STD_BILLING_RULE: SkuBillingBasisRule = SkuBillingBasisRule {
    sku_type: "RPT_STD",
    default_event_type: Some("one_time_charge"),
    cycle_event_type: None,
    usage_event_type: None,
    payment_trigger: "lock_after_report_order_created",
    delivery_trigger: "report_generated_and_downloadable",
    acceptance_trigger: "buyer_accept_or_timeout_acceptance",
    billing_trigger: "bill_once_after_report_acceptance",
    settlement_cycle: "t_plus_1_once",
    refund_entry: "refund_if_report_not_generated_or_rejected",
    periodic_settlement_cycle: None,
    refund_placeholder_entry: None,
    refund_placeholder_event_type: None,
    compensation_entry: "compensate_on_critical_report_defect",
    dispute_freeze_trigger: "freeze_on_report_quality_dispute",
    resume_settlement_trigger: "resume_on_review_passed_dispute_closed",
};

pub fn sku_billing_basis_rule_for_sku(sku_type: &str) -> Option<SkuBillingBasisRule> {
    match sku_type {
        "FILE_STD" => Some(FILE_STD_BILLING_RULE),
        "FILE_SUB" => Some(FILE_SUB_BILLING_RULE),
        "SHARE_RO" => Some(SHARE_RO_BILLING_RULE),
        "API_SUB" => Some(API_SUB_BILLING_RULE),
        "API_PPU" => Some(API_PPU_BILLING_RULE),
        "QRY_LITE" => Some(QRY_LITE_BILLING_RULE),
        "SBX_STD" => Some(SBX_STD_BILLING_RULE),
        "RPT_STD" => Some(RPT_STD_BILLING_RULE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::sku_billing_basis_rule_for_sku;

    #[test]
    fn non_api_default_rules_cover_remaining_v1_standard_skus() {
        for sku in [
            "FILE_STD", "FILE_SUB", "SHARE_RO", "QRY_LITE", "SBX_STD", "RPT_STD",
        ] {
            let rule = sku_billing_basis_rule_for_sku(sku)
                .unwrap_or_else(|| panic!("missing sku billing rule for {sku}"));
            assert_eq!(rule.sku_type, sku);
            assert!(!rule.billing_trigger.is_empty());
            assert!(!rule.refund_entry.is_empty());
            assert!(!rule.settlement_cycle.is_empty());
        }
    }

    #[test]
    fn share_ro_supports_opening_charge_plus_cycle_extension_after_bil026() {
        let rule = sku_billing_basis_rule_for_sku("SHARE_RO").expect("share ro rule");
        assert_eq!(rule.default_event_type, Some("one_time_charge"));
        assert_eq!(rule.cycle_event_type, Some("recurring_charge"));
        assert_eq!(rule.billing_trigger, "bill_once_on_grant_effective");
        assert_eq!(rule.refund_entry, "refund_if_grant_not_effective");
        assert_eq!(
            rule.refund_placeholder_entry,
            Some("placeholder_on_share_revoke_or_scope_cutoff")
        );
    }

    #[test]
    fn recurring_default_rules_match_subscription_style_skus() {
        let file_sub = sku_billing_basis_rule_for_sku("FILE_SUB").expect("file sub");
        let sbx_std = sku_billing_basis_rule_for_sku("SBX_STD").expect("sbx std");
        assert_eq!(file_sub.default_event_type, Some("recurring_charge"));
        assert_eq!(sbx_std.default_event_type, Some("recurring_charge"));
    }
}
