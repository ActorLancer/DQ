mod storage_gateway;
mod watermark_policy;

pub use storage_gateway::{
    StorageGatewayAccessAudit, StorageGatewayDownloadRestriction, StorageGatewayIntegrity,
    StorageGatewayObjectLocator, StorageGatewaySnapshot, StorageGatewayWatermarkPolicy,
};
pub(crate) use watermark_policy::{
    build_watermark_placeholder_patch, derive_storage_gateway_watermark_policy,
    merge_snapshot_patch,
};
