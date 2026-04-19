use config::{ProviderMode, RuntimeConfig};
use kernel::{AppResult, new_external_readable_id};
use provider_kit::{ProviderBackend, SignatureRequest, build_signing_provider};

#[derive(Debug, Clone)]
pub struct ContractSigningResult {
    pub provider_mode: String,
    pub provider_kind: String,
    pub provider_ref: String,
}

pub async fn sign_contract_with_provider(
    contract_id: &str,
    signer_id: &str,
) -> AppResult<ContractSigningResult> {
    let runtime = RuntimeConfig::from_env()?;
    let backend = match runtime.provider {
        ProviderMode::Mock => ProviderBackend::Mock,
        ProviderMode::Real => ProviderBackend::Real,
    };
    let provider = build_signing_provider(backend);
    let raw_ref = provider
        .sign(SignatureRequest {
            document_id: contract_id.to_string(),
            signer_party_id: signer_id.to_string(),
        })
        .await?;

    Ok(ContractSigningResult {
        provider_mode: runtime.provider.as_str().to_string(),
        provider_kind: provider.kind().to_string(),
        provider_ref: format!(
            "{}:{}:{}",
            raw_ref,
            contract_id,
            new_external_readable_id("sign")
        ),
    })
}
