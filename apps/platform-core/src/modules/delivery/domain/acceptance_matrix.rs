#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptanceFlowKind {
    ManualDeliverySignoff,
    EnablementAndFirstValidUse,
    ExecutionResultReady,
    EnablementSuccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptanceTriggerRule {
    pub sku_type: &'static str,
    pub accept_template_code: &'static str,
    pub flow_kind: AcceptanceFlowKind,
    pub manual_delivery_branch: Option<&'static str>,
    pub pre_acceptance_states: &'static [&'static str],
    pub pending_acceptance_status: &'static str,
    pub accepted_states: &'static [&'static str],
    pub trigger_evidence: &'static [&'static str],
}

const FILE_STD_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "FILE_STD",
    accept_template_code: "ACCEPT_FILE_V1",
    flow_kind: AcceptanceFlowKind::ManualDeliverySignoff,
    manual_delivery_branch: Some("file"),
    pre_acceptance_states: &["delivered"],
    pending_acceptance_status: "pending_acceptance",
    accepted_states: &["accepted"],
    trigger_evidence: &["delivery.file.commit", "delivery.accept", "delivery.reject"],
};

const FILE_SUB_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "FILE_SUB",
    accept_template_code: "ACCEPT_FILE_SUB_V1",
    flow_kind: AcceptanceFlowKind::ManualDeliverySignoff,
    manual_delivery_branch: Some("file"),
    pre_acceptance_states: &["delivered"],
    pending_acceptance_status: "pending_acceptance",
    accepted_states: &["accepted"],
    trigger_evidence: &["delivery.file.commit", "delivery.accept", "delivery.reject"],
};

const RPT_STD_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "RPT_STD",
    accept_template_code: "ACCEPT_REPORT_V1",
    flow_kind: AcceptanceFlowKind::ManualDeliverySignoff,
    manual_delivery_branch: Some("report"),
    pre_acceptance_states: &["report_delivered"],
    pending_acceptance_status: "in_progress",
    accepted_states: &["accepted"],
    trigger_evidence: &[
        "delivery.report.commit",
        "delivery.accept",
        "delivery.reject",
    ],
};

const API_SUB_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "API_SUB",
    accept_template_code: "ACCEPT_API_SUB_V1",
    flow_kind: AcceptanceFlowKind::EnablementAndFirstValidUse,
    manual_delivery_branch: None,
    pre_acceptance_states: &["api_bound", "api_key_issued", "api_trial_active"],
    pending_acceptance_status: "not_started",
    accepted_states: &["active"],
    trigger_evidence: &["delivery.api.enable", "api.first_success_call"],
};

const API_PPU_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "API_PPU",
    accept_template_code: "ACCEPT_API_PPU_V1",
    flow_kind: AcceptanceFlowKind::EnablementAndFirstValidUse,
    manual_delivery_branch: None,
    pre_acceptance_states: &["api_authorized", "quota_ready"],
    pending_acceptance_status: "not_started",
    accepted_states: &["usage_active"],
    trigger_evidence: &["delivery.api.enable", "api.first_success_call"],
};

const QRY_LITE_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "QRY_LITE",
    accept_template_code: "ACCEPT_QUERY_LITE_V1",
    flow_kind: AcceptanceFlowKind::ExecutionResultReady,
    manual_delivery_branch: None,
    pre_acceptance_states: &["template_authorized", "params_validated"],
    pending_acceptance_status: "not_started",
    accepted_states: &["query_executed", "result_available"],
    trigger_evidence: &["delivery.template_query.use", "query.result_downloadable"],
};

const SHARE_RO_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "SHARE_RO",
    accept_template_code: "ACCEPT_SHARE_RO_V1",
    flow_kind: AcceptanceFlowKind::EnablementSuccess,
    manual_delivery_branch: None,
    pre_acceptance_states: &["share_enabled"],
    pending_acceptance_status: "not_started",
    accepted_states: &["share_granted", "shared_active"],
    trigger_evidence: &["delivery.share.enable"],
};

const SBX_STD_RULE: AcceptanceTriggerRule = AcceptanceTriggerRule {
    sku_type: "SBX_STD",
    accept_template_code: "ACCEPT_SANDBOX_V1",
    flow_kind: AcceptanceFlowKind::EnablementSuccess,
    manual_delivery_branch: None,
    pre_acceptance_states: &["workspace_enabled"],
    pending_acceptance_status: "not_started",
    accepted_states: &["seat_issued", "sandbox_executed", "result_limited_exported"],
    trigger_evidence: &["delivery.sandbox.enable"],
};

pub const ACCEPTANCE_TRIGGER_RULES: &[AcceptanceTriggerRule] = &[
    FILE_STD_RULE,
    FILE_SUB_RULE,
    RPT_STD_RULE,
    API_SUB_RULE,
    API_PPU_RULE,
    QRY_LITE_RULE,
    SHARE_RO_RULE,
    SBX_STD_RULE,
];

pub fn acceptance_trigger_rule(sku_type: &str) -> Option<&'static AcceptanceTriggerRule> {
    ACCEPTANCE_TRIGGER_RULES
        .iter()
        .find(|rule| rule.sku_type == sku_type)
}

pub fn manual_acceptance_delivery_branch(sku_type: &str) -> Option<&'static str> {
    acceptance_trigger_rule(sku_type).and_then(|rule| rule.manual_delivery_branch)
}

pub fn is_manual_acceptance_state(sku_type: &str, current_state: &str) -> bool {
    acceptance_trigger_rule(sku_type)
        .map(|rule| rule.pre_acceptance_states.contains(&current_state))
        .unwrap_or(false)
}

pub fn is_accepted_state(sku_type: &str, current_state: &str) -> bool {
    acceptance_trigger_rule(sku_type)
        .map(|rule| rule.accepted_states.contains(&current_state))
        .unwrap_or(false)
}

pub fn expected_acceptance_status_for_state(
    sku_type: &str,
    current_state: &str,
) -> Option<&'static str> {
    let rule = acceptance_trigger_rule(sku_type)?;
    if rule.accepted_states.contains(&current_state) {
        return Some("accepted");
    }
    if rule.pre_acceptance_states.contains(&current_state) {
        return Some(rule.pending_acceptance_status);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        ACCEPTANCE_TRIGGER_RULES, AcceptanceFlowKind, acceptance_trigger_rule,
        expected_acceptance_status_for_state, is_accepted_state, is_manual_acceptance_state,
    };

    #[test]
    fn frozen_v1_acceptance_matrix_covers_all_standard_skus() {
        let sku_types = ACCEPTANCE_TRIGGER_RULES
            .iter()
            .map(|rule| rule.sku_type)
            .collect::<Vec<_>>();
        assert_eq!(
            sku_types,
            vec![
                "FILE_STD", "FILE_SUB", "RPT_STD", "API_SUB", "API_PPU", "QRY_LITE", "SHARE_RO",
                "SBX_STD",
            ]
        );
    }

    #[test]
    fn manual_signoff_rules_remain_file_and_report_only() {
        for sku in ["FILE_STD", "FILE_SUB", "RPT_STD"] {
            assert_eq!(
                acceptance_trigger_rule(sku).map(|rule| rule.flow_kind),
                Some(AcceptanceFlowKind::ManualDeliverySignoff)
            );
            assert!(is_manual_acceptance_state(
                sku,
                acceptance_trigger_rule(sku)
                    .expect("rule")
                    .pre_acceptance_states[0]
            ));
        }
        for sku in ["API_SUB", "API_PPU", "QRY_LITE", "SHARE_RO", "SBX_STD"] {
            assert!(!is_manual_acceptance_state(sku, "buyer_locked"));
        }
    }

    #[test]
    fn auto_accept_rules_match_frozen_stage_expectations() {
        assert_eq!(
            expected_acceptance_status_for_state("SHARE_RO", "share_granted"),
            Some("accepted")
        );
        assert!(is_accepted_state("SBX_STD", "seat_issued"));
        assert_eq!(
            expected_acceptance_status_for_state("API_SUB", "api_key_issued"),
            Some("not_started")
        );
        assert!(is_accepted_state("API_SUB", "active"));
        assert!(is_accepted_state("API_PPU", "usage_active"));
        assert!(is_accepted_state("QRY_LITE", "query_executed"));
    }

    #[test]
    fn acceptance_template_codes_match_frozen_baseline() {
        let expected_templates = vec![
            ("FILE_STD", "ACCEPT_FILE_V1"),
            ("FILE_SUB", "ACCEPT_FILE_SUB_V1"),
            ("RPT_STD", "ACCEPT_REPORT_V1"),
            ("API_SUB", "ACCEPT_API_SUB_V1"),
            ("API_PPU", "ACCEPT_API_PPU_V1"),
            ("QRY_LITE", "ACCEPT_QUERY_LITE_V1"),
            ("SHARE_RO", "ACCEPT_SHARE_RO_V1"),
            ("SBX_STD", "ACCEPT_SANDBOX_V1"),
        ];
        let actual_templates = ACCEPTANCE_TRIGGER_RULES
            .iter()
            .map(|rule| (rule.sku_type, rule.accept_template_code))
            .collect::<Vec<_>>();
        assert_eq!(actual_templates, expected_templates);
    }
}
