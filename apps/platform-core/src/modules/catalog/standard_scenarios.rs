use crate::modules::catalog::domain::StandardScenarioTemplateView;
use serde_json::json;

pub fn standard_scenario_templates() -> Vec<StandardScenarioTemplateView> {
    vec![
        StandardScenarioTemplateView {
            scenario_code: "S1".to_string(),
            scenario_name: "工业设备运行指标 API 订阅".to_string(),
            primary_sku: "API_SUB".to_string(),
            supplementary_skus: vec!["API_PPU".to_string()],
            product_template: json!({"product_type":"service_product","delivery_type":"api_subscription","category":"industry_iot"}),
            metadata_template: json!({"industry":"industrial_manufacturing","use_cases":["设备稼动率","能耗监控"],"data_classification":"P1"}),
            contract_template: "CONTRACT_API_SUB_V1".to_string(),
            acceptance_template: "ACCEPT_API_SUB_V1".to_string(),
            refund_template: "REFUND_API_SUB_V1".to_string(),
            review_sample: json!({"action_name":"approve","action_reason":"api_schema_and_sla_verified"}),
        },
        StandardScenarioTemplateView {
            scenario_code: "S2".to_string(),
            scenario_name: "工业质量与产线日报文件包交付".to_string(),
            primary_sku: "FILE_STD".to_string(),
            supplementary_skus: vec!["FILE_SUB".to_string()],
            product_template: json!({"product_type":"data_product","delivery_type":"file_download","category":"industrial_quality"}),
            metadata_template: json!({"industry":"industrial_manufacturing","use_cases":["质量日报","产线巡检"],"data_classification":"P1"}),
            contract_template: "CONTRACT_FILE_V1".to_string(),
            acceptance_template: "ACCEPT_FILE_V1".to_string(),
            refund_template: "REFUND_FILE_V1".to_string(),
            review_sample: json!({"action_name":"approve","action_reason":"file_hash_and_preview_checked"}),
        },
        StandardScenarioTemplateView {
            scenario_code: "S3".to_string(),
            scenario_name: "供应链协同查询沙箱".to_string(),
            primary_sku: "SBX_STD".to_string(),
            supplementary_skus: vec!["SHARE_RO".to_string()],
            product_template: json!({"product_type":"service_product","delivery_type":"sandbox","category":"supply_chain"}),
            metadata_template: json!({"industry":"industrial_manufacturing","use_cases":["履约分析","库存协同"],"data_classification":"P2"}),
            contract_template: "CONTRACT_SANDBOX_V1".to_string(),
            acceptance_template: "ACCEPT_SANDBOX_V1".to_string(),
            refund_template: "REFUND_SANDBOX_V1".to_string(),
            review_sample: json!({"action_name":"approve","action_reason":"sandbox_guardrail_verified"}),
        },
        StandardScenarioTemplateView {
            scenario_code: "S4".to_string(),
            scenario_name: "零售门店经营分析 API / 报告订阅".to_string(),
            primary_sku: "API_SUB".to_string(),
            supplementary_skus: vec!["RPT_STD".to_string()],
            product_template: json!({"product_type":"service_product","delivery_type":"api_subscription","category":"retail_ops"}),
            metadata_template: json!({"industry":"retail","use_cases":["客流分析","销售结构"],"data_classification":"P1"}),
            contract_template: "CONTRACT_API_SUB_V1".to_string(),
            acceptance_template: "ACCEPT_API_SUB_V1".to_string(),
            refund_template: "REFUND_API_SUB_V1".to_string(),
            review_sample: json!({"action_name":"approve","action_reason":"api_report_combo_check_passed"}),
        },
        StandardScenarioTemplateView {
            scenario_code: "S5".to_string(),
            scenario_name: "商圈/门店选址查询服务".to_string(),
            primary_sku: "QRY_LITE".to_string(),
            supplementary_skus: vec!["RPT_STD".to_string()],
            product_template: json!({"product_type":"service_product","delivery_type":"query_template","category":"retail_location"}),
            metadata_template: json!({"industry":"retail","use_cases":["选址评分","商圈画像"],"data_classification":"P1"}),
            contract_template: "CONTRACT_QUERY_LITE_V1".to_string(),
            acceptance_template: "ACCEPT_QUERY_LITE_V1".to_string(),
            refund_template: "REFUND_QUERY_LITE_V1".to_string(),
            review_sample: json!({"action_name":"approve","action_reason":"template_boundary_validated"}),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::standard_scenario_templates;

    #[test]
    fn includes_five_frozen_standard_scenarios() {
        let templates = standard_scenario_templates();
        assert_eq!(templates.len(), 5);
        assert!(templates.iter().any(|s| s.scenario_code == "S1"));
        assert!(templates.iter().any(|s| s.scenario_code == "S5"));
    }
}
