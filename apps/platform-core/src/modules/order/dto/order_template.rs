use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTemplateView {
    pub template_code: String,
    pub scenario_code: String,
    pub scenario_name: String,
    pub industry_code: String,
    pub primary_sku: String,
    pub supplementary_skus: Vec<String>,
    pub contract_template: String,
    pub acceptance_template: String,
    pub refund_template: String,
    pub workflow_steps: Vec<String>,
    pub order_draft: Value,
}
