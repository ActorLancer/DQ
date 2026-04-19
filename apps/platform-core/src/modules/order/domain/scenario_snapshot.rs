use crate::modules::catalog::standard_scenarios::standard_scenario_templates;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioSkuSnapshot {
    pub scenario_code: String,
    pub scenario_name: String,
    pub selected_sku_id: String,
    pub selected_sku_code: String,
    pub selected_sku_type: String,
    pub selected_sku_role: String,
    pub primary_sku: String,
    pub supplementary_skus: Vec<String>,
    pub contract_template: String,
    pub acceptance_template: String,
    pub refund_template: String,
    pub per_sku_snapshot_required: bool,
    pub multi_sku_requires_independent_contract_authorization_settlement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioSnapshotResolveError {
    NoStandardScenarioForSku {
        sku_type: String,
    },
    UnknownScenarioCode {
        scenario_code: String,
    },
    ScenarioSkuMismatch {
        scenario_code: String,
        sku_type: String,
    },
    AmbiguousScenarioForSku {
        sku_type: String,
        scenario_codes: Vec<String>,
    },
}

impl ScenarioSnapshotResolveError {
    pub fn message(&self) -> String {
        match self {
            Self::NoStandardScenarioForSku { sku_type } => {
                format!(
                    "ORDER_CREATE_FORBIDDEN: no frozen standard scenario found for sku_type `{sku_type}`"
                )
            }
            Self::UnknownScenarioCode { scenario_code } => {
                format!(
                    "ORDER_CREATE_FORBIDDEN: scenario_code `{scenario_code}` is not in frozen standard scenarios"
                )
            }
            Self::ScenarioSkuMismatch {
                scenario_code,
                sku_type,
            } => {
                format!(
                    "ORDER_CREATE_FORBIDDEN: scenario_code `{scenario_code}` does not include sku_type `{sku_type}`"
                )
            }
            Self::AmbiguousScenarioForSku {
                sku_type,
                scenario_codes,
            } => format!(
                "ORDER_CREATE_FORBIDDEN: scenario_code is required for sku_type `{sku_type}` because it belongs to multiple frozen scenarios: {}",
                scenario_codes.join(",")
            ),
        }
    }
}

pub fn resolve_standard_scenario_snapshot(
    sku_id: &str,
    sku_code: &str,
    sku_type: &str,
    requested_scenario_code: Option<&str>,
    product_metadata: &Value,
) -> Result<ScenarioSkuSnapshot, ScenarioSnapshotResolveError> {
    let preferred_scenario_code = requested_scenario_code.or_else(|| {
        product_metadata
            .get("standard_scenario_code")
            .and_then(Value::as_str)
            .or_else(|| {
                product_metadata
                    .get("scenario_code")
                    .and_then(Value::as_str)
            })
    });

    let templates = standard_scenario_templates();
    let matching_templates = templates
        .iter()
        .filter(|template| {
            template.primary_sku == sku_type
                || template
                    .supplementary_skus
                    .iter()
                    .any(|supplementary| supplementary == sku_type)
        })
        .collect::<Vec<_>>();

    if matching_templates.is_empty() {
        return Err(ScenarioSnapshotResolveError::NoStandardScenarioForSku {
            sku_type: sku_type.to_string(),
        });
    }

    let template = if let Some(scenario_code) = preferred_scenario_code {
        templates
            .iter()
            .find(|candidate| candidate.scenario_code == scenario_code)
            .ok_or_else(|| ScenarioSnapshotResolveError::UnknownScenarioCode {
                scenario_code: scenario_code.to_string(),
            })?
    } else if matching_templates.len() == 1 {
        matching_templates[0]
    } else {
        return Err(ScenarioSnapshotResolveError::AmbiguousScenarioForSku {
            sku_type: sku_type.to_string(),
            scenario_codes: matching_templates
                .iter()
                .map(|candidate| candidate.scenario_code.clone())
                .collect(),
        });
    };

    let selected_sku_role = if template.primary_sku == sku_type {
        "primary"
    } else if template
        .supplementary_skus
        .iter()
        .any(|supplementary| supplementary == sku_type)
    {
        "supplementary"
    } else {
        return Err(ScenarioSnapshotResolveError::ScenarioSkuMismatch {
            scenario_code: template.scenario_code.clone(),
            sku_type: sku_type.to_string(),
        });
    };

    Ok(ScenarioSkuSnapshot {
        scenario_code: template.scenario_code.clone(),
        scenario_name: template.scenario_name.clone(),
        selected_sku_id: sku_id.to_string(),
        selected_sku_code: sku_code.to_string(),
        selected_sku_type: sku_type.to_string(),
        selected_sku_role: selected_sku_role.to_string(),
        primary_sku: template.primary_sku.clone(),
        supplementary_skus: template.supplementary_skus.clone(),
        contract_template: template.contract_template.clone(),
        acceptance_template: template.acceptance_template.clone(),
        refund_template: template.refund_template.clone(),
        per_sku_snapshot_required: true,
        multi_sku_requires_independent_contract_authorization_settlement: true,
    })
}

#[cfg(test)]
mod tests {
    use super::{ScenarioSnapshotResolveError, resolve_standard_scenario_snapshot};
    use serde_json::json;

    #[test]
    fn resolves_unique_sku_without_explicit_scenario_code() {
        let snapshot = resolve_standard_scenario_snapshot(
            "order-sku-id",
            "TRADE-SKU-FILE",
            "FILE_STD",
            None,
            &json!({}),
        )
        .expect("FILE_STD should map to unique scenario");
        assert_eq!(snapshot.scenario_code, "S2");
        assert_eq!(snapshot.selected_sku_role, "primary");
    }

    #[test]
    fn requires_explicit_scenario_for_ambiguous_sku() {
        let err = resolve_standard_scenario_snapshot(
            "order-sku-id",
            "TRADE-SKU-API",
            "API_SUB",
            None,
            &json!({}),
        )
        .expect_err("API_SUB belongs to multiple scenarios");
        assert_eq!(
            err,
            ScenarioSnapshotResolveError::AmbiguousScenarioForSku {
                sku_type: "API_SUB".to_string(),
                scenario_codes: vec!["S1".to_string(), "S4".to_string()],
            }
        );
    }

    #[test]
    fn resolves_supplementary_sku_with_explicit_scenario() {
        let snapshot = resolve_standard_scenario_snapshot(
            "order-sku-id",
            "TRADE-SKU-RPT",
            "RPT_STD",
            Some("S5"),
            &json!({}),
        )
        .expect("RPT_STD should resolve with scenario code");
        assert_eq!(snapshot.scenario_code, "S5");
        assert_eq!(snapshot.selected_sku_role, "supplementary");
        assert_eq!(snapshot.primary_sku, "QRY_LITE");
    }

    #[test]
    fn rejects_scenario_sku_mismatch() {
        let err = resolve_standard_scenario_snapshot(
            "order-sku-id",
            "TRADE-SKU-FILE",
            "FILE_STD",
            Some("S1"),
            &json!({}),
        )
        .expect_err("S1 does not include FILE_STD");
        assert_eq!(
            err,
            ScenarioSnapshotResolveError::ScenarioSkuMismatch {
                scenario_code: "S1".to_string(),
                sku_type: "FILE_STD".to_string(),
            }
        );
    }

    #[test]
    fn falls_back_to_product_metadata_hint() {
        let snapshot = resolve_standard_scenario_snapshot(
            "order-sku-id",
            "TRADE-SKU-API",
            "API_SUB",
            None,
            &json!({"standard_scenario_code":"S4"}),
        )
        .expect("metadata hint should disambiguate API_SUB");
        assert_eq!(snapshot.scenario_code, "S4");
    }
}
