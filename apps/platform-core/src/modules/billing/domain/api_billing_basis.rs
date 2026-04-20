use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ApiBillingBasisView {
    pub sku_type: String,
    pub base_event_type: Option<String>,
    pub usage_event_type: Option<String>,
    pub cycle_period: Option<String>,
    pub included_units: Option<String>,
    pub overage_policy: Option<String>,
    pub usage_meter_source: Option<String>,
    pub success_only: bool,
    pub latest_usage_call_count: String,
    pub latest_usage_units: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiBillingBasisRule {
    pub sku_type: &'static str,
    pub base_event_type: Option<&'static str>,
    pub usage_event_type: Option<&'static str>,
    pub default_cycle_period: Option<&'static str>,
    pub default_overage_policy: Option<&'static str>,
    pub usage_meter_source: Option<&'static str>,
    pub success_only: bool,
}

pub const API_SUB_BILLING_RULE: ApiBillingBasisRule = ApiBillingBasisRule {
    sku_type: "API_SUB",
    base_event_type: Some("recurring_charge"),
    usage_event_type: Some("usage_charge"),
    default_cycle_period: Some("monthly"),
    default_overage_policy: Some("metered"),
    usage_meter_source: Some("delivery.api_usage_log"),
    success_only: true,
};

pub const API_PPU_BILLING_RULE: ApiBillingBasisRule = ApiBillingBasisRule {
    sku_type: "API_PPU",
    base_event_type: None,
    usage_event_type: Some("usage_charge"),
    default_cycle_period: Some("per_call"),
    default_overage_policy: Some("per_call"),
    usage_meter_source: Some("delivery.api_usage_log"),
    success_only: true,
};

pub fn api_billing_basis_rule_for_sku(sku_type: &str) -> Option<ApiBillingBasisRule> {
    match sku_type {
        "API_SUB" => Some(API_SUB_BILLING_RULE),
        "API_PPU" => Some(API_PPU_BILLING_RULE),
        _ => None,
    }
}
