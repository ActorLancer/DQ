//! cargo test -p http@0.1.0

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn pagination_has_default_and_clamp() {
        let p = kernel::Pagination::from_query(Some(kernel::PaginationQuery {
            page: Some(0),
            page_size: Some(9999),
        }));
        assert_eq!(p.page, 1);
        assert_eq!(p.page_size, 200);
        assert_eq!(p.offset(), 0);
    }

    #[test]
    fn list_query_builds_from_parts() {
        let q = kernel::ListQuery::new(
            Some(kernel::PaginationQuery {
                page: Some(2),
                page_size: Some(25),
            }),
            Some(kernel::FilterQuery {
                keyword: Some("order".to_string()),
                status: Some("open".to_string()),
                sort_by: Some("created_at".to_string()),
                sort_order: Some("desc".to_string()),
            }),
        );
        assert_eq!(q.pagination.offset(), 25);
        assert_eq!(q.filter.status.as_deref(), Some("open"));
    }

    #[test]
    fn idempotency_key_prefers_standard_header() {
        let mut headers = HeaderMap::new();
        headers.insert("idempotency-key", HeaderValue::from_static("idem-001"));
        headers.insert("x-idempotency-key", HeaderValue::from_static("legacy-001"));
        assert_eq!(
            resolve_idempotency_key(&headers, "req-001"),
            "idem-001".to_string()
        );
    }

    #[test]
    fn idempotency_key_falls_back_to_request_id() {
        let headers = HeaderMap::new();
        assert_eq!(
            resolve_idempotency_key(&headers, "req-007"),
            "req-007".to_string()
        );
    }

    #[test]
    fn trace_links_use_default_ports() {
        let links = build_trace_links();
        assert_eq!(links.grafana, "http://localhost:3000");
        assert_eq!(links.loki, "http://localhost:3100");
        assert_eq!(links.tempo, "http://localhost:3200");
        assert_eq!(links.keycloak, "http://localhost:8081");
        assert_eq!(links.minio_console, "http://localhost:9001");
        assert_eq!(links.opensearch, "http://localhost:9200");
    }

    #[test]
    fn dev_overview_feed_is_capped() {
        for i in 0..(DEV_OVERVIEW_WINDOW + 3) {
            record_outbox_event(format!("evt-{i}"), "dtp.outbox.domain-events", "pending");
        }
        let overview = build_dev_overview();
        assert_eq!(overview.recent_outbox.len(), DEV_OVERVIEW_WINDOW);
        assert_eq!(
            overview
                .recent_outbox
                .first()
                .map(|it| it.event_id.as_str()),
            Some("evt-12")
        );
    }

    #[test]
    fn metrics_path_normalizes_numeric_and_uuid_like_segments() {
        assert_eq!(
            normalize_metrics_path(
                "/api/v1/orders/123/payments/550e8400-e29b-41d4-a716-446655440000"
            ),
            "/api/v1/orders/{id}/payments/{id}"
        );
    }

    #[test]
    fn looks_like_dynamic_segment_detects_ids() {
        assert!(looks_like_dynamic_path_segment("12345"));
        assert!(looks_like_dynamic_path_segment(
            "550e8400-e29b-41d4-a716-446655440000"
        ));
        assert!(!looks_like_dynamic_path_segment("projection-gaps"));
    }
}
