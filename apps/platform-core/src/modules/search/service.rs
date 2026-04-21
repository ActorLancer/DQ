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

pub fn is_allowed(role: &str, permission: SearchPermission) -> bool {
    match permission {
        SearchPermission::PortalRead => matches!(role, "platform_admin" | "buyer_operator"),
        SearchPermission::SyncRead
        | SearchPermission::ReindexExecute
        | SearchPermission::AliasManage
        | SearchPermission::CacheInvalidate
        | SearchPermission::RankingRead
        | SearchPermission::RankingManage => matches!(role, "platform_admin"),
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
