#[cfg(test)]
mod tests {
    use crate::*;
    use std::time::Duration;

    async fn mock_payment_live_ready() -> bool {
        let mode = std::env::var("MOCK_PAYMENT_ADAPTER_MODE")
            .unwrap_or_else(|_| "stub".to_string())
            .to_ascii_lowercase();
        if mode != "live" {
            eprintln!(
                "skip live_mock_payment_adapter_hits_three_mock_paths: \
                 MOCK_PAYMENT_ADAPTER_MODE is not set to live"
            );
            return false;
        }

        let base_url = std::env::var("MOCK_PAYMENT_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8089".to_string());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("build mock payment health client");
        match client
            .get(format!("{}/health/ready", base_url.trim_end_matches('/')))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => true,
            Ok(resp) => {
                eprintln!(
                    "skip live_mock_payment_adapter_hits_three_mock_paths: \
                     mock payment readiness probe returned status {}",
                    resp.status()
                );
                false
            }
            Err(err) => {
                eprintln!(
                    "skip live_mock_payment_adapter_hits_three_mock_paths: \
                     mock payment provider is not reachable: {err}"
                );
                false
            }
        }
    }

    #[tokio::test]
    async fn providers_have_mock_and_real_entrypoints() {
        let mock_kyc = build_kyc_provider(ProviderBackend::Mock);
        let real_kyc = build_kyc_provider(ProviderBackend::Real);
        assert_eq!(mock_kyc.kind(), "mock");
        assert_eq!(real_kyc.kind(), "real");

        let request = KycCheckRequest {
            party_id: "pty-1".to_string(),
            jurisdiction: "CN".to_string(),
        };
        assert_eq!(
            mock_kyc.verify(request.clone()).await.unwrap(),
            "mock-kyc-ok"
        );
        assert_eq!(real_kyc.verify(request).await.unwrap(), "real-kyc-ok");

        let mock_sign = build_signing_provider(ProviderBackend::Mock);
        let real_sign = build_signing_provider(ProviderBackend::Real);
        assert_eq!(
            mock_sign
                .sign(SignatureRequest {
                    document_id: "doc-1".to_string(),
                    signer_party_id: "pty-1".to_string(),
                })
                .await
                .unwrap(),
            "mock-signing-ok"
        );
        assert_eq!(
            real_sign
                .sign(SignatureRequest {
                    document_id: "doc-2".to_string(),
                    signer_party_id: "pty-2".to_string(),
                })
                .await
                .unwrap(),
            "real-signing-ok"
        );

        let mock_pay = build_payment_provider(ProviderBackend::Mock);
        let real_pay = build_payment_provider(ProviderBackend::Real);
        assert_eq!(
            mock_pay
                .create_intent(PaymentRequest {
                    order_id: "ord-1".to_string(),
                    amount_minor: 100,
                    currency: "CNY".to_string(),
                })
                .await
                .unwrap(),
            "mock-payment-ok"
        );
        assert_eq!(
            real_pay
                .create_intent(PaymentRequest {
                    order_id: "ord-2".to_string(),
                    amount_minor: 200,
                    currency: "USD".to_string(),
                })
                .await
                .unwrap(),
            "real-payment-ok"
        );

        let mock_notify = build_notification_provider(ProviderBackend::Mock);
        let real_notify = build_notification_provider(ProviderBackend::Real);
        assert_eq!(
            mock_notify
                .send(NotificationRequest {
                    template_code: "TPL_A".to_string(),
                    receiver: "u1@example.com".to_string(),
                })
                .await
                .unwrap(),
            "mock-notify-ok"
        );
        assert_eq!(
            real_notify
                .send(NotificationRequest {
                    template_code: "TPL_B".to_string(),
                    receiver: "u2@example.com".to_string(),
                })
                .await
                .unwrap(),
            "real-notify-ok"
        );

        let mock_fabric = build_fabric_writer_provider(ProviderBackend::Mock);
        let real_fabric = build_fabric_writer_provider(ProviderBackend::Real);
        assert_eq!(
            mock_fabric
                .write(FabricWriteRequest {
                    channel: "ch1".to_string(),
                    key: "k1".to_string(),
                    value_json: "{}".to_string(),
                })
                .await
                .unwrap(),
            "mock-fabric-ok"
        );
        assert_eq!(
            real_fabric
                .write(FabricWriteRequest {
                    channel: "ch2".to_string(),
                    key: "k2".to_string(),
                    value_json: "{\"ok\":true}".to_string(),
                })
                .await
                .unwrap(),
            "real-fabric-ok"
        );
    }

    #[tokio::test]
    async fn mock_payment_adapter_supports_three_scenarios_in_stub_mode() {
        let provider = build_payment_provider(ProviderBackend::Mock);
        let success = provider
            .simulate_webhook("pay-1", MockPaymentScenario::Success)
            .await
            .unwrap();
        let fail = provider
            .simulate_webhook("pay-2", MockPaymentScenario::Fail)
            .await
            .unwrap();
        let timeout = provider
            .simulate_webhook("pay-3", MockPaymentScenario::Timeout)
            .await
            .unwrap();
        assert_eq!(success.event_type, "payment.succeeded");
        assert_eq!(fail.event_type, "payment.failed");
        assert_eq!(timeout.event_type, "payment.timeout");
    }

    #[tokio::test]
    #[ignore = "requires local mock-payment container and MOCK_PAYMENT_ADAPTER_MODE=live"]
    async fn live_mock_payment_adapter_hits_three_mock_paths() {
        if !mock_payment_live_ready().await {
            return;
        }

        let provider = build_payment_provider(ProviderBackend::Mock);
        let success = provider
            .simulate_webhook("pay-live-1", MockPaymentScenario::Success)
            .await
            .unwrap();
        let fail = provider
            .simulate_webhook("pay-live-2", MockPaymentScenario::Fail)
            .await
            .unwrap();
        let timeout = provider
            .simulate_webhook("pay-live-3", MockPaymentScenario::Timeout)
            .await
            .unwrap();
        assert_eq!(success.http_status_code, Some(200));
        assert_eq!(fail.http_status_code, Some(402));
        assert_eq!(timeout.http_status_code, None);
    }
}
