use crate::modules::catalog::domain::{is_supported_trade_mode, is_trade_mode_compatible_with_sku};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogPermission {
    ProductDraftWrite,
}

pub fn is_allowed(role: &str, permission: CatalogPermission) -> bool {
    match permission {
        CatalogPermission::ProductDraftWrite => {
            matches!(role, "platform_admin" | "tenant_admin" | "tenant_operator")
        }
    }
}

pub fn is_valid_sku_trade_mode_pair(sku_type: &str, trade_mode: &str) -> bool {
    is_supported_trade_mode(trade_mode) && is_trade_mode_compatible_with_sku(sku_type, trade_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_draft_write_role_matrix() {
        assert!(is_allowed(
            "platform_admin",
            CatalogPermission::ProductDraftWrite
        ));
        assert!(is_allowed(
            "tenant_admin",
            CatalogPermission::ProductDraftWrite
        ));
        assert!(is_allowed(
            "tenant_operator",
            CatalogPermission::ProductDraftWrite
        ));
        assert!(!is_allowed(
            "developer",
            CatalogPermission::ProductDraftWrite
        ));
    }

    #[test]
    fn sku_trade_mode_pair_validation() {
        assert!(is_valid_sku_trade_mode_pair("FILE_STD", "snapshot_sale"));
        assert!(is_valid_sku_trade_mode_pair("API_PPU", "api_pay_per_use"));
        assert!(!is_valid_sku_trade_mode_pair("API_PPU", "api_subscription"));
        assert!(!is_valid_sku_trade_mode_pair("FILE_STD", "unknown_mode"));
    }
}
