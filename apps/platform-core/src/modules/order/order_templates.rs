use crate::modules::catalog::standard_scenarios::standard_scenario_templates;
use crate::modules::order::dto::OrderTemplateView;
use serde_json::json;

pub fn standard_order_templates() -> Vec<OrderTemplateView> {
    standard_scenario_templates()
        .into_iter()
        .map(|scenario| {
            let primary_sku = scenario.primary_sku.clone();
            let supplementary_skus = scenario.supplementary_skus.clone();
            let contract_template = scenario.contract_template.clone();
            let acceptance_template = scenario.acceptance_template.clone();
            let refund_template = scenario.refund_template.clone();
            let (industry_code, template_code, workflow_steps, primary_flow_code) =
                match scenario.scenario_code.as_str() {
                    "S1" => (
                        "industrial_manufacturing",
                        "ORDER_TEMPLATE_API_SUB_V1",
                        vec![
                            "listed".to_string(),
                            "contract_confirm".to_string(),
                            "payment_lock_first_cycle".to_string(),
                            "bind_application".to_string(),
                            "issue_api_credential".to_string(),
                            "first_success_call".to_string(),
                            "cycle_billing".to_string(),
                        ],
                        "api_subscription",
                    ),
                    "S2" => (
                        "industrial_manufacturing",
                        "ORDER_TEMPLATE_FILE_STD_V1",
                        vec![
                            "listed".to_string(),
                            "contract_confirm".to_string(),
                            "payment_lock_full".to_string(),
                            "issue_download_ticket".to_string(),
                            "download_and_verify".to_string(),
                            "acceptance".to_string(),
                            "settlement".to_string(),
                        ],
                        "file_snapshot",
                    ),
                    "S3" => (
                        "industrial_manufacturing",
                        "ORDER_TEMPLATE_SBX_STD_V1",
                        vec![
                            "listed".to_string(),
                            "contract_confirm".to_string(),
                            "payment_lock".to_string(),
                            "enable_sandbox".to_string(),
                            "restricted_query".to_string(),
                            "restricted_export".to_string(),
                            "settlement".to_string(),
                        ],
                        "sandbox_query",
                    ),
                    "S4" => (
                        "retail",
                        "ORDER_TEMPLATE_API_SUB_RPT_V1",
                        vec![
                            "listed".to_string(),
                            "contract_confirm".to_string(),
                            "payment_lock_first_cycle".to_string(),
                            "bind_application".to_string(),
                            "issue_api_credential".to_string(),
                            "first_success_call".to_string(),
                            "report_delivery_optional".to_string(),
                            "cycle_billing".to_string(),
                        ],
                        "api_subscription",
                    ),
                    "S5" => (
                        "retail",
                        "ORDER_TEMPLATE_QRY_LITE_V1",
                        vec![
                            "listed".to_string(),
                            "contract_confirm".to_string(),
                            "payment_lock".to_string(),
                            "grant_template_query".to_string(),
                            "execute_template_query".to_string(),
                            "restricted_result_export".to_string(),
                            "settlement".to_string(),
                        ],
                        "template_query",
                    ),
                    _ => (
                        "unknown",
                        "ORDER_TEMPLATE_UNKNOWN_V1",
                        vec!["listed".to_string()],
                        "unknown",
                    ),
                };

            OrderTemplateView {
                template_code: template_code.to_string(),
                scenario_code: scenario.scenario_code,
                scenario_name: scenario.scenario_name,
                industry_code: industry_code.to_string(),
                primary_sku: primary_sku.clone(),
                supplementary_skus: supplementary_skus.clone(),
                contract_template: contract_template.clone(),
                acceptance_template: acceptance_template.clone(),
                refund_template: refund_template.clone(),
                workflow_steps,
                order_draft: json!({
                    "primary_flow_code": primary_flow_code,
                    "primary_sku": primary_sku,
                    "supplementary_skus": supplementary_skus,
                    "per_sku_snapshot_required": true,
                    "contract_template": contract_template,
                    "acceptance_template": acceptance_template,
                    "refund_template": refund_template,
                    "multi_sku_requires_independent_contract_authorization_settlement": true
                }),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::standard_order_templates;

    #[test]
    fn includes_five_frozen_order_templates_and_all_v1_skus() {
        let templates = standard_order_templates();
        assert_eq!(templates.len(), 5);

        let mut sku_set = std::collections::BTreeSet::new();
        for template in &templates {
            sku_set.insert(template.primary_sku.as_str().to_string());
            for sku in &template.supplementary_skus {
                sku_set.insert(sku.clone());
            }
            assert!(template.order_draft["per_sku_snapshot_required"].as_bool() == Some(true));
        }

        let expected = [
            "API_SUB", "API_PPU", "FILE_STD", "FILE_SUB", "SBX_STD", "SHARE_RO", "QRY_LITE",
            "RPT_STD",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(sku_set, expected);
    }
}
