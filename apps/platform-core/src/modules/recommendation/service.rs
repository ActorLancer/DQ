#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationPermission {
    PortalRead,
    PlacementRead,
    PlacementManage,
    RankingRead,
    RankingManage,
    RebuildExecute,
}

pub fn is_allowed(role: &str, permission: RecommendationPermission) -> bool {
    match permission {
        RecommendationPermission::PortalRead => matches!(
            role,
            "platform_admin" | "buyer_operator" | "seller_operator" | "tenant_admin"
        ),
        RecommendationPermission::PlacementRead
        | RecommendationPermission::PlacementManage
        | RecommendationPermission::RankingRead
        | RecommendationPermission::RankingManage
        | RecommendationPermission::RebuildExecute => matches!(role, "platform_admin"),
    }
}

pub fn needs_step_up(permission: RecommendationPermission) -> bool {
    matches!(
        permission,
        RecommendationPermission::PlacementManage
            | RecommendationPermission::RankingManage
            | RecommendationPermission::RebuildExecute
    )
}
