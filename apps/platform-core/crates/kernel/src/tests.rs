#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn container_roundtrip() {
        let c = ServiceContainer::default();
        c.insert::<String>("abc".to_string()).await;
        let value = c.get::<String>().await.expect("value should exist");
        assert_eq!(value.as_str(), "abc");
    }

    #[test]
    fn error_code_document_is_compatible() {
        let doc = include_str!("../../../../../docs/01-architecture/error-codes.md");
        validate_error_code_document(doc)
            .expect("error codes doc should include required prefixes");
    }

    #[test]
    fn utc_timestamp_is_monotonic_non_negative() {
        let t1 = UtcTimestampMs::now();
        let t2 = UtcTimestampMs::now();
        assert!(t1.0 >= 0);
        assert!(t2.0 >= t1.0);
    }

    #[test]
    fn entity_id_parse_roundtrip() {
        let id = EntityId::new();
        let raw = id.to_string();
        let parsed = EntityId::parse(&raw).expect("parse entity id");
        assert_eq!(parsed, id);
    }

    #[test]
    fn external_readable_id_has_prefix() {
        let id = new_external_readable_id("ord");
        assert!(id.starts_with("ORD-"));
        assert_eq!(id.split('-').count(), 3);
    }

    #[test]
    fn in_process_event_bus_roundtrip() {
        let bus = InProcessEventBus::new(8);
        let mut rx = bus.subscribe();
        let event = DomainEventEnvelope {
            event_name: "order.created".to_string(),
            aggregate_type: "order".to_string(),
            aggregate_id: "ord-1".to_string(),
            payload_json: "{\"order_id\":\"ord-1\"}".to_string(),
            occurred_at_utc_ms: UtcTimestampMs::now().0,
        };
        bus.publish(event.clone()).expect("publish event");
        let got = rx.try_recv().expect("receive event");
        assert_eq!(got, event);
    }

    #[test]
    fn pagination_has_default_and_clamp() {
        let p = Pagination::from_query(Some(PaginationQuery {
            page: Some(0),
            page_size: Some(9999),
        }));
        assert_eq!(p.page, 1);
        assert_eq!(p.page_size, 200);
        assert_eq!(p.offset(), 0);
    }

    #[test]
    fn list_query_builds_from_parts() {
        let q = ListQuery::new(
            Some(PaginationQuery {
                page: Some(2),
                page_size: Some(25),
            }),
            Some(FilterQuery {
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
    fn error_response_uses_explicit_code_without_message_override() {
        let response = ErrorResponse {
            code: "FILE_STD_TRANSITION_FORBIDDEN".to_string(),
            message: "FILE_STD_TRANSITION_FORBIDDEN: invalid transition".to_string(),
            request_id: Some("req-kernel-embedded".to_string()),
        };

        let json = serde_json::to_value(&response).expect("serialize error response");
        assert_eq!(json["code"].as_str(), Some("FILE_STD_TRANSITION_FORBIDDEN"));
        assert_eq!(json["message"].as_str(), Some(response.message.as_str()));
        assert_eq!(json["request_id"].as_str(), Some("req-kernel-embedded"));
        assert_eq!(json["details"], serde_json::json!({}));
    }

    #[test]
    fn error_response_keeps_generic_code_without_business_prefix() {
        let response = ErrorResponse {
            code: "FORBIDDEN".to_string(),
            message: "permission denied".to_string(),
            request_id: Some("req-kernel-generic".to_string()),
        };

        let json = serde_json::to_value(&response).expect("serialize error response");
        assert_eq!(json["code"].as_str(), Some("FORBIDDEN"));
        assert_eq!(json["request_id"].as_str(), Some("req-kernel-generic"));
        assert_eq!(json["details"], serde_json::json!({}));
    }
}
