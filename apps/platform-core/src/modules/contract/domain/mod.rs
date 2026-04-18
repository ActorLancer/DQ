use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContractExpectationSnapshot {
    pub contract_template_id: Option<String>,
    pub expected_term_days: Option<i32>,
    pub expected_sla_tier: Option<String>,
    pub custom_terms_note: Option<String>,
}
