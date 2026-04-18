#[cfg(test)]
mod tests {
    use crate::modules::catalog::domain::{BindTemplateRequest, PatchUsagePolicyRequest};
    use crate::modules::catalog::repository::PostgresCatalogRepository;
    use serde_json::json;
    use tokio_postgres::NoTls;

    fn live_db_enabled() -> bool {
        std::env::var("CATALOG_DB_SMOKE").ok().as_deref() == Some("1")
    }

    fn db_dsn() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string()
        })
    }

    #[tokio::test]
    async fn repository_bind_template_to_sku_smoke() {
        if !live_db_enabled() {
            return;
        }
        let (mut client, connection) = tokio_postgres::connect(&db_dsn(), NoTls)
            .await
            .expect("connect database");
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let tx = client.transaction().await.expect("begin transaction");
        let row = tx
            .query_opt(
                "SELECT sku.sku_id::text, t.template_id::text
                 FROM catalog.product_sku sku
                 JOIN contract.template_definition t
                   ON sku.sku_type = ANY(t.applicable_sku_types)
                  AND t.status = 'active'
                 LIMIT 1",
                &[],
            )
            .await
            .expect("load sku/template pair");
        let Some(row) = row else {
            tx.rollback().await.expect("rollback");
            return;
        };
        let sku_id: String = row.get(0);
        let template_id: String = row.get(1);
        let payload = BindTemplateRequest {
            template_id: template_id.clone(),
            binding_type: Some("contract".to_string()),
        };
        PostgresCatalogRepository::bind_template_to_sku(&tx, &sku_id, &payload)
            .await
            .expect("bind template");
        let metadata_row = tx
            .query_one(
                "SELECT metadata->>'draft_template_id'
                 FROM catalog.product_sku
                 WHERE sku_id = $1::text::uuid",
                &[&sku_id],
            )
            .await
            .expect("query sku metadata");
        let bound_template: Option<String> = metadata_row.get(0);
        assert_eq!(bound_template.as_deref(), Some(template_id.as_str()));
        tx.rollback().await.expect("rollback");
    }

    #[tokio::test]
    async fn repository_patch_usage_policy_smoke() {
        if !live_db_enabled() {
            return;
        }
        let (mut client, connection) = tokio_postgres::connect(&db_dsn(), NoTls)
            .await
            .expect("connect database");
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let tx = client.transaction().await.expect("begin transaction");
        let owner_row = tx
            .query_one(
                "SELECT org_id::text FROM core.organization ORDER BY created_at ASC LIMIT 1",
                &[],
            )
            .await
            .expect("load owner org");
        let owner_org_id: String = owner_row.get(0);
        let inserted = tx
            .query_one(
                "INSERT INTO contract.usage_policy (
                   owner_org_id, policy_name, status
                 ) VALUES (
                   $1::text::uuid, 'CAT-021 smoke', 'draft'
                 )
                 RETURNING policy_id::text",
                &[&owner_org_id],
            )
            .await
            .expect("insert usage policy");
        let policy_id: String = inserted.get(0);
        let payload = PatchUsagePolicyRequest {
            policy_name: Some("CAT-021 smoke updated".to_string()),
            subject_constraints: Some(json!({"tenant_scope":"self"})),
            usage_constraints: Some(json!({"allow":["access","result_get"]})),
            time_constraints: None,
            region_constraints: None,
            output_constraints: None,
            exportable: Some(false),
            status: Some("active".to_string()),
        };
        let updated = PostgresCatalogRepository::patch_usage_policy(&tx, &policy_id, &payload)
            .await
            .expect("patch policy")
            .expect("policy exists");
        assert_eq!(updated.policy_id, policy_id);
        assert_eq!(updated.policy_name, "CAT-021 smoke updated");
        assert_eq!(updated.status, "active");
        tx.rollback().await.expect("rollback");
    }
}
