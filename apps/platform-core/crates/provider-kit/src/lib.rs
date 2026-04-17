use async_trait::async_trait;
use kernel::{AppError, AppResult, new_external_readable_id};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderBackend {
    Mock,
    Real,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KycCheckRequest {
    pub party_id: String,
    pub jurisdiction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignatureRequest {
    pub document_id: String,
    pub signer_party_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentRequest {
    pub order_id: String,
    pub amount_minor: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MockPaymentScenario {
    Success,
    Fail,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockPaymentWebhookEvent {
    pub provider_event_id: String,
    pub payment_intent_id: String,
    pub scenario: MockPaymentScenario,
    pub event_type: String,
    pub provider_status: String,
    pub http_status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationRequest {
    pub template_code: String,
    pub receiver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FabricWriteRequest {
    pub channel: String,
    pub key: String,
    pub value_json: String,
}

#[async_trait]
pub trait KycProvider: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn verify(&self, request: KycCheckRequest) -> AppResult<String>;
}

#[async_trait]
pub trait SigningProvider: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn sign(&self, request: SignatureRequest) -> AppResult<String>;
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn create_intent(&self, request: PaymentRequest) -> AppResult<String>;
    async fn simulate_webhook(
        &self,
        payment_intent_id: &str,
        scenario: MockPaymentScenario,
    ) -> AppResult<MockPaymentWebhookEvent>;
}

#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn send(&self, request: NotificationRequest) -> AppResult<String>;
}

#[async_trait]
pub trait FabricWriterProvider: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn write(&self, request: FabricWriteRequest) -> AppResult<String>;
}

macro_rules! define_provider_impl {
    ($name:ident, $trait_name:ident, $method:ident, $req:ty, $prefix:literal, $kind:literal) => {
        #[derive(Debug, Default, Clone)]
        pub struct $name;

        #[async_trait]
        impl $trait_name for $name {
            fn kind(&self) -> &'static str {
                $kind
            }

            async fn $method(&self, _request: $req) -> AppResult<String> {
                Ok(format!("{}-ok", $prefix))
            }
        }
    };
}

define_provider_impl!(
    MockKycProvider,
    KycProvider,
    verify,
    KycCheckRequest,
    "mock-kyc",
    "mock"
);
define_provider_impl!(
    RealKycProvider,
    KycProvider,
    verify,
    KycCheckRequest,
    "real-kyc",
    "real"
);
define_provider_impl!(
    MockSigningProvider,
    SigningProvider,
    sign,
    SignatureRequest,
    "mock-signing",
    "mock"
);
define_provider_impl!(
    RealSigningProvider,
    SigningProvider,
    sign,
    SignatureRequest,
    "real-signing",
    "real"
);
define_provider_impl!(
    MockNotificationProvider,
    NotificationProvider,
    send,
    NotificationRequest,
    "mock-notify",
    "mock"
);
define_provider_impl!(
    RealNotificationProvider,
    NotificationProvider,
    send,
    NotificationRequest,
    "real-notify",
    "real"
);
define_provider_impl!(
    MockFabricWriterProvider,
    FabricWriterProvider,
    write,
    FabricWriteRequest,
    "mock-fabric",
    "mock"
);
define_provider_impl!(
    RealFabricWriterProvider,
    FabricWriterProvider,
    write,
    FabricWriteRequest,
    "real-fabric",
    "real"
);

#[derive(Debug, Default, Clone)]
pub struct MockPaymentProvider;

#[derive(Debug, Default, Clone)]
pub struct RealPaymentProvider;

#[async_trait]
impl PaymentProvider for MockPaymentProvider {
    fn kind(&self) -> &'static str {
        "mock"
    }

    async fn create_intent(&self, _request: PaymentRequest) -> AppResult<String> {
        Ok("mock-payment-ok".to_string())
    }

    async fn simulate_webhook(
        &self,
        payment_intent_id: &str,
        scenario: MockPaymentScenario,
    ) -> AppResult<MockPaymentWebhookEvent> {
        let mode = std::env::var("MOCK_PAYMENT_ADAPTER_MODE")
            .unwrap_or_else(|_| "stub".to_string())
            .to_ascii_lowercase();
        if mode != "live" {
            return Ok(build_mock_event(payment_intent_id, scenario, None));
        }

        let base_url = std::env::var("MOCK_PAYMENT_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8089".to_string());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|err| AppError::Startup(format!("mock payment client init failed: {err}")))?;
        let endpoint = match scenario {
            MockPaymentScenario::Success => "/mock/payment/charge/success",
            MockPaymentScenario::Fail => "/mock/payment/charge/fail",
            MockPaymentScenario::Timeout => "/mock/payment/charge/timeout",
        };
        let url = format!("{}{}", base_url.trim_end_matches('/'), endpoint);
        let response = client.post(&url).send().await;

        match (scenario, response) {
            (MockPaymentScenario::Timeout, Err(err)) if err.is_timeout() => {
                Ok(build_mock_event(payment_intent_id, scenario, None))
            }
            (_, Ok(resp)) => Ok(build_mock_event(
                payment_intent_id,
                scenario,
                Some(resp.status().as_u16()),
            )),
            (_, Err(err)) => Err(AppError::Startup(format!(
                "mock payment scenario invoke failed: {err}"
            ))),
        }
    }
}

#[async_trait]
impl PaymentProvider for RealPaymentProvider {
    fn kind(&self) -> &'static str {
        "real"
    }

    async fn create_intent(&self, _request: PaymentRequest) -> AppResult<String> {
        Ok("real-payment-ok".to_string())
    }

    async fn simulate_webhook(
        &self,
        payment_intent_id: &str,
        scenario: MockPaymentScenario,
    ) -> AppResult<MockPaymentWebhookEvent> {
        Ok(build_mock_event(payment_intent_id, scenario, None))
    }
}

fn build_mock_event(
    payment_intent_id: &str,
    scenario: MockPaymentScenario,
    http_status_code: Option<u16>,
) -> MockPaymentWebhookEvent {
    let (event_type, provider_status) = match scenario {
        MockPaymentScenario::Success => ("payment.succeeded", "succeeded"),
        MockPaymentScenario::Fail => ("payment.failed", "failed"),
        MockPaymentScenario::Timeout => ("payment.timeout", "timeout"),
    };
    MockPaymentWebhookEvent {
        provider_event_id: new_external_readable_id("mockpayevt"),
        payment_intent_id: payment_intent_id.to_string(),
        scenario,
        event_type: event_type.to_string(),
        provider_status: provider_status.to_string(),
        http_status_code,
    }
}

pub fn build_kyc_provider(backend: ProviderBackend) -> Arc<dyn KycProvider> {
    match backend {
        ProviderBackend::Mock => Arc::new(MockKycProvider),
        ProviderBackend::Real => Arc::new(RealKycProvider),
    }
}

pub fn build_signing_provider(backend: ProviderBackend) -> Arc<dyn SigningProvider> {
    match backend {
        ProviderBackend::Mock => Arc::new(MockSigningProvider),
        ProviderBackend::Real => Arc::new(RealSigningProvider),
    }
}

pub fn build_payment_provider(backend: ProviderBackend) -> Arc<dyn PaymentProvider> {
    match backend {
        ProviderBackend::Mock => Arc::new(MockPaymentProvider),
        ProviderBackend::Real => Arc::new(RealPaymentProvider),
    }
}

pub fn build_notification_provider(backend: ProviderBackend) -> Arc<dyn NotificationProvider> {
    match backend {
        ProviderBackend::Mock => Arc::new(MockNotificationProvider),
        ProviderBackend::Real => Arc::new(RealNotificationProvider),
    }
}

pub fn build_fabric_writer_provider(backend: ProviderBackend) -> Arc<dyn FabricWriterProvider> {
    match backend {
        ProviderBackend::Mock => Arc::new(MockFabricWriterProvider),
        ProviderBackend::Real => Arc::new(RealFabricWriterProvider),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
