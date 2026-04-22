#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchPermission {
    PortalRead,
    SyncRead,
    ReindexExecute,
    AliasManage,
    CacheInvalidate,
    RankingRead,
    RankingManage,
}

impl SearchPermission {
    pub fn permission_code(self) -> &'static str {
        match self {
            SearchPermission::PortalRead => "portal.search.read",
            SearchPermission::SyncRead => "ops.search_sync.read",
            SearchPermission::ReindexExecute => "ops.search_reindex.execute",
            SearchPermission::AliasManage => "ops.search_alias.manage",
            SearchPermission::CacheInvalidate => "ops.search_cache.invalidate",
            SearchPermission::RankingRead => "ops.search_ranking.read",
            SearchPermission::RankingManage => "ops.search_ranking.manage",
        }
    }

    pub fn forbidden_code(self) -> &'static str {
        match self {
            SearchPermission::ReindexExecute => "SEARCH_REINDEX_FORBIDDEN",
            SearchPermission::AliasManage => "SEARCH_ALIAS_SWITCH_FORBIDDEN",
            SearchPermission::CacheInvalidate => "SEARCH_CACHE_INVALIDATE_FORBIDDEN",
            _ => "IAM_UNAUTHORIZED",
        }
    }
}

pub fn is_allowed(roles: &[String], permission: SearchPermission) -> bool {
    roles
        .iter()
        .any(|role| role_has_permission(role.as_str(), permission))
}

pub fn first_matching_role(roles: &[String], permission: SearchPermission) -> Option<String> {
    roles
        .iter()
        .find(|role| role_has_permission(role.as_str(), permission))
        .cloned()
}

fn role_has_permission(role: &str, permission: SearchPermission) -> bool {
    match role {
        "buyer_operator" => matches!(permission, SearchPermission::PortalRead),
        "platform_admin" => matches!(
            permission,
            SearchPermission::PortalRead
                | SearchPermission::SyncRead
                | SearchPermission::ReindexExecute
                | SearchPermission::AliasManage
                | SearchPermission::CacheInvalidate
                | SearchPermission::RankingRead
                | SearchPermission::RankingManage
        ),
        "platform_audit_security" => matches!(
            permission,
            SearchPermission::SyncRead | SearchPermission::CacheInvalidate
        ),
        _ => false,
    }
}

pub fn needs_step_up(permission: SearchPermission) -> bool {
    matches!(
        permission,
        SearchPermission::ReindexExecute
            | SearchPermission::AliasManage
            | SearchPermission::RankingManage
    )
}
