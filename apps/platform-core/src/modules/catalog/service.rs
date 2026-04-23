use crate::modules::catalog::domain::{is_supported_trade_mode, is_trade_mode_compatible_with_sku};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogPermission {
    ProductDraftWrite,
    ProductList,
    ProductRead,
    SellerProfileRead,
    TemplateBind,
    PolicyUpdate,
    RawIngestWrite,
    ProductSubmit,
    ReviewWrite,
    ProductSuspend,
    RiskProductFreeze,
}

pub fn is_allowed(role: &str, permission: CatalogPermission) -> bool {
    match permission {
        CatalogPermission::ProductDraftWrite => {
            matches!(role, "platform_admin" | "tenant_admin" | "seller_operator")
        }
        CatalogPermission::ProductList => {
            matches!(role, "platform_admin" | "tenant_admin" | "seller_operator")
        }
        CatalogPermission::ProductRead => {
            matches!(
                role,
                "platform_admin"
                    | "platform_reviewer"
                    | "tenant_admin"
                    | "seller_operator"
                    | "buyer_operator"
                    | "tenant_developer"
            )
        }
        CatalogPermission::SellerProfileRead => {
            matches!(
                role,
                "platform_admin"
                    | "tenant_admin"
                    | "seller_operator"
                    | "buyer_operator"
                    | "tenant_audit_readonly"
            )
        }
        CatalogPermission::TemplateBind => {
            matches!(role, "platform_admin" | "tenant_admin" | "seller_operator")
        }
        CatalogPermission::PolicyUpdate => {
            matches!(role, "platform_admin" | "tenant_admin")
        }
        CatalogPermission::RawIngestWrite => {
            matches!(role, "platform_admin" | "tenant_admin" | "seller_operator")
        }
        CatalogPermission::ProductSubmit => {
            matches!(role, "platform_admin" | "tenant_admin" | "seller_operator")
        }
        CatalogPermission::ReviewWrite => matches!(role, "platform_admin" | "platform_reviewer"),
        CatalogPermission::ProductSuspend => {
            matches!(role, "platform_admin" | "tenant_admin" | "seller_operator")
        }
        CatalogPermission::RiskProductFreeze => {
            matches!(role, "platform_admin" | "platform_risk_settlement")
        }
    }
}

pub fn is_valid_sku_trade_mode_pair(sku_type: &str, trade_mode: &str) -> bool {
    is_supported_trade_mode(trade_mode) && is_trade_mode_compatible_with_sku(sku_type, trade_mode)
}

pub fn is_valid_listing_status(status: &str) -> bool {
    matches!(
        status,
        "draft" | "pending_review" | "listed" | "delisted" | "frozen"
    )
}

pub fn can_transition_listing_status(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("draft", "pending_review")
            | ("pending_review", "listed")
            | ("pending_review", "draft")
            | ("pending_review", "frozen")
            | ("listed", "delisted")
            | ("listed", "frozen")
            | ("delisted", "listed")
            | ("delisted", "frozen")
            | ("frozen", "delisted")
    )
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
            "seller_operator",
            CatalogPermission::ProductDraftWrite
        ));
        assert!(!is_allowed(
            "tenant_developer",
            CatalogPermission::ProductDraftWrite
        ));
        assert!(is_allowed("platform_admin", CatalogPermission::ProductList));
        assert!(is_allowed("tenant_admin", CatalogPermission::ProductList));
        assert!(is_allowed(
            "seller_operator",
            CatalogPermission::ProductList
        ));
        assert!(!is_allowed(
            "buyer_operator",
            CatalogPermission::ProductList
        ));
    }

    #[test]
    fn sku_trade_mode_pair_validation() {
        assert!(is_valid_sku_trade_mode_pair("FILE_STD", "snapshot_sale"));
        assert!(is_valid_sku_trade_mode_pair("API_PPU", "api_pay_per_use"));
        assert!(!is_valid_sku_trade_mode_pair("API_PPU", "api_subscription"));
        assert!(!is_valid_sku_trade_mode_pair("FILE_STD", "unknown_mode"));
    }

    #[test]
    fn raw_ingest_write_role_matrix() {
        assert!(is_allowed(
            "platform_admin",
            CatalogPermission::RawIngestWrite
        ));
        assert!(is_allowed(
            "tenant_admin",
            CatalogPermission::RawIngestWrite
        ));
        assert!(is_allowed(
            "seller_operator",
            CatalogPermission::RawIngestWrite
        ));
        assert!(!is_allowed(
            "tenant_developer",
            CatalogPermission::RawIngestWrite
        ));
    }

    #[test]
    fn read_role_matrix() {
        assert!(is_allowed("platform_admin", CatalogPermission::ProductRead));
        assert!(is_allowed(
            "platform_reviewer",
            CatalogPermission::ProductRead
        ));
        assert!(is_allowed("tenant_admin", CatalogPermission::ProductRead));
        assert!(is_allowed("buyer_operator", CatalogPermission::ProductRead));
        assert!(is_allowed(
            "seller_operator",
            CatalogPermission::ProductRead
        ));
        assert!(is_allowed(
            "tenant_developer",
            CatalogPermission::ProductRead
        ));
        assert!(!is_allowed(
            "tenant_app_identity",
            CatalogPermission::ProductRead
        ));
        assert!(is_allowed(
            "platform_admin",
            CatalogPermission::SellerProfileRead
        ));
        assert!(is_allowed(
            "tenant_admin",
            CatalogPermission::SellerProfileRead
        ));
        assert!(is_allowed(
            "buyer_operator",
            CatalogPermission::SellerProfileRead
        ));
        assert!(is_allowed(
            "seller_operator",
            CatalogPermission::SellerProfileRead
        ));
        assert!(is_allowed(
            "tenant_audit_readonly",
            CatalogPermission::SellerProfileRead
        ));
        assert!(!is_allowed(
            "tenant_developer",
            CatalogPermission::SellerProfileRead
        ));
        assert!(is_allowed(
            "platform_admin",
            CatalogPermission::TemplateBind
        ));
        assert!(is_allowed("tenant_admin", CatalogPermission::TemplateBind));
        assert!(is_allowed(
            "seller_operator",
            CatalogPermission::TemplateBind
        ));
        assert!(!is_allowed(
            "buyer_operator",
            CatalogPermission::TemplateBind
        ));
        assert!(is_allowed(
            "platform_admin",
            CatalogPermission::PolicyUpdate
        ));
        assert!(is_allowed("tenant_admin", CatalogPermission::PolicyUpdate));
        assert!(!is_allowed(
            "seller_operator",
            CatalogPermission::PolicyUpdate
        ));
    }

    #[test]
    fn listing_status_machine_is_frozen() {
        assert!(is_valid_listing_status("pending_review"));
        assert!(can_transition_listing_status("draft", "pending_review"));
        assert!(can_transition_listing_status("pending_review", "listed"));
        assert!(can_transition_listing_status("listed", "delisted"));
        assert!(can_transition_listing_status("listed", "frozen"));
        assert!(!can_transition_listing_status("draft", "listed"));
        assert!(!can_transition_listing_status("frozen", "listed"));
    }
}
