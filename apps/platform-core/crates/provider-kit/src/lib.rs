use async_trait::async_trait;
use kernel::AppResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

define_provider_impl!(MockKycProvider, KycProvider, verify, KycCheckRequest, "mock-kyc", "mock");
define_provider_impl!(RealKycProvider, KycProvider, verify, KycCheckRequest, "real-kyc", "real");
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
    MockPaymentProvider,
    PaymentProvider,
    create_intent,
    PaymentRequest,
    "mock-payment",
    "mock"
);
define_provider_impl!(
    RealPaymentProvider,
    PaymentProvider,
    create_intent,
    PaymentRequest,
    "real-payment",
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
        assert_eq!(mock_kyc.verify(request.clone()).await.unwrap(), "mock-kyc-ok");
        assert_eq!(real_kyc.verify(request).await.unwrap(), "real-kyc-ok");
    }
}
