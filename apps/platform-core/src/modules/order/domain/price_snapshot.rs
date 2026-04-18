use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettlementTermsSnapshot {
    pub settlement_basis: String,
    pub settlement_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaxTermsSnapshot {
    pub tax_policy: String,
    pub tax_code: String,
    pub tax_inclusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderPriceSnapshot {
    pub product_id: String,
    pub sku_id: String,
    pub sku_code: String,
    pub sku_type: String,
    pub pricing_mode: String,
    pub unit_price: String,
    pub currency_code: String,
    pub billing_mode: String,
    pub refund_mode: String,
    pub settlement_terms: SettlementTermsSnapshot,
    pub tax_terms: TaxTermsSnapshot,
    pub captured_at: String,
    pub source: String,
}

pub fn derive_settlement_basis(billing_mode: &str, pricing_mode: &str) -> String {
    match (billing_mode, pricing_mode) {
        ("one_time", _) => "one_time_final".to_string(),
        ("subscription", _) => "periodic_cycle".to_string(),
        ("pay_per_use", _) => "usage_metered".to_string(),
        (_, "subscription") => "periodic_cycle".to_string(),
        (_, "pay_per_use") => "usage_metered".to_string(),
        _ => "manual_v1_default".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::derive_settlement_basis;

    #[test]
    fn settlement_basis_prefers_billing_mode() {
        assert_eq!(
            derive_settlement_basis("one_time", "subscription"),
            "one_time_final"
        );
        assert_eq!(
            derive_settlement_basis("subscription", "one_time"),
            "periodic_cycle"
        );
        assert_eq!(
            derive_settlement_basis("pay_per_use", "one_time"),
            "usage_metered"
        );
    }

    #[test]
    fn settlement_basis_falls_back_to_pricing_mode_then_default() {
        assert_eq!(
            derive_settlement_basis("unknown", "subscription"),
            "periodic_cycle"
        );
        assert_eq!(
            derive_settlement_basis("unknown", "unknown"),
            "manual_v1_default"
        );
    }
}
