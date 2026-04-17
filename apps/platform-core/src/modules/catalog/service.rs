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
}
