#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn crate_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn repo_root() -> PathBuf {
        crate_root()
            .parent()
            .and_then(Path::parent)
            .expect("repo root")
            .to_path_buf()
    }

    fn read_json(path: &Path) -> Value {
        let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
        serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
    }

    #[test]
    fn dlv026_standard_delivery_demo_assets_are_frozen_and_complete() {
        let repo_root = repo_root();
        let manifest_path = repo_root
            .join("apps/platform-core/src/modules/delivery/tests/fixtures/dlv026/manifest.json");
        let manifest = read_json(&manifest_path);
        assert_eq!(manifest["generated_for_task"].as_str(), Some("DLV-026"));

        let standard_manifest_path =
            repo_root.join("fixtures/local/standard-scenarios-manifest.json");
        let standard_manifest = read_json(&standard_manifest_path);

        let mut expected = BTreeMap::new();
        for scenario in standard_manifest["scenarios"]
            .as_array()
            .expect("standard scenarios")
        {
            expected.insert(
                scenario["scenario_id"]
                    .as_str()
                    .expect("scenario id")
                    .to_string(),
                (
                    scenario["primary_sku_type"]
                        .as_str()
                        .expect("primary sku")
                        .to_string(),
                    scenario["optional_sku_types"]
                        .as_array()
                        .expect("optional skus")
                        .iter()
                        .map(|sku| sku.as_str().expect("sku").to_string())
                        .collect::<Vec<_>>(),
                ),
            );
        }

        let scenarios = manifest["scenarios"]
            .as_array()
            .expect("manifest scenarios");
        assert_eq!(scenarios.len(), 5, "DLV-026 should cover S1~S5 exactly");

        for scenario in scenarios {
            let scenario_id = scenario["scenario_id"].as_str().expect("scenario_id");
            let (primary, optional) = expected
                .get(scenario_id)
                .unwrap_or_else(|| panic!("unexpected scenario {scenario_id}"));
            assert_eq!(
                scenario["primary_sku_type"].as_str(),
                Some(primary.as_str())
            );
            let optional_json = scenario["supplementary_sku_types"]
                .as_array()
                .expect("supplementary skus")
                .iter()
                .map(|sku| sku.as_str().expect("sku").to_string())
                .collect::<Vec<_>>();
            assert_eq!(&optional_json, optional);

            let script_path =
                repo_root.join(scenario["demo_script"].as_str().expect("demo script path"));
            assert!(
                script_path.is_file(),
                "missing script {}",
                script_path.display()
            );
            let script_raw = fs::read_to_string(&script_path)
                .unwrap_or_else(|_| panic!("read {}", script_path.display()));
            assert!(
                script_raw.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"),
                "demo script should be strict bash: {}",
                script_path.display()
            );

            let objects = scenario["minimal_delivery_objects"]
                .as_array()
                .expect("minimal delivery objects");
            assert!(
                !objects.is_empty(),
                "scenario {scenario_id} must contain at least one minimal delivery object"
            );
            for object in objects {
                let fixture_path =
                    repo_root.join(object["fixture"].as_str().expect("fixture path"));
                assert!(
                    fixture_path.is_file(),
                    "missing fixture {}",
                    fixture_path.display()
                );
                let fixture = read_json(&fixture_path);
                assert_eq!(fixture["scenario_id"].as_str(), Some(scenario_id));
                assert_eq!(
                    fixture["sku_type"].as_str(),
                    object["sku_type"].as_str(),
                    "fixture/object sku mismatch for {}",
                    fixture_path.display()
                );
                assert!(
                    object["api_path"]
                        .as_str()
                        .unwrap_or_default()
                        .starts_with("/api/v1/")
                );
                assert!(matches!(
                    object["http_method"].as_str(),
                    Some("GET") | Some("POST")
                ));
            }
        }
    }
}
