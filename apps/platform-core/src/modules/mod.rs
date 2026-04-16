pub mod access;
pub mod audit;
pub mod authorization;
pub mod billing;
pub mod catalog;
pub mod consistency;
pub mod contract;
pub mod contract_meta;
pub mod delivery;
pub mod developer;
pub mod dispute;
pub mod fairness;
pub mod iam;
pub mod inquiry;
pub mod integration;
pub mod listing;
pub mod ops;
pub mod order;
pub mod party;
pub mod provider_ops;
pub mod recommendation;
pub mod review;
pub mod search;

#[cfg(test)]
mod tests {
    use std::path::Path;

    const CORE010_REQUIRED_MODULES: &[&str] = &[
        "access",
        "audit",
        "authorization",
        "billing",
        "catalog",
        "consistency",
        "contract",
        "contract_meta",
        "delivery",
        "developer",
        "dispute",
        "iam",
        "listing",
        "ops",
        "order",
        "party",
        "recommendation",
        "review",
        "search",
    ];
    const TEMPLATE_DIRS: &[&str] = &[
        "api",
        "application",
        "domain",
        "repo",
        "dto",
        "events",
        "tests",
    ];

    #[test]
    fn core010_required_modules_follow_template_layout() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/modules");
        for module in CORE010_REQUIRED_MODULES {
            for dir in TEMPLATE_DIRS {
                let path = root.join(module).join(dir);
                assert!(
                    path.is_dir(),
                    "module template missing: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn legacy_recommend_module_directory_is_absent() {
        let legacy = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/modules/recommend");
        assert!(
            !legacy.exists(),
            "legacy module directory should be removed: {}",
            legacy.display()
        );
    }
}
