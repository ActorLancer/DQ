#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationPermission {
    PortalRead,
    PlacementRead,
    PlacementManage,
    RankingRead,
    RankingManage,
    RebuildExecute,
}

impl RecommendationPermission {
    pub fn permission_code(self) -> &'static str {
        match self {
            RecommendationPermission::PortalRead => "portal.recommendation.read",
            RecommendationPermission::PlacementRead => "ops.recommendation.read",
            RecommendationPermission::PlacementManage => "ops.recommendation.manage",
            RecommendationPermission::RankingRead => "ops.recommendation.read",
            RecommendationPermission::RankingManage => "ops.recommendation.manage",
            RecommendationPermission::RebuildExecute => "ops.recommend_rebuild.execute",
        }
    }

    pub fn forbidden_code(self) -> &'static str {
        match self {
            RecommendationPermission::PortalRead
            | RecommendationPermission::PlacementRead
            | RecommendationPermission::RankingRead => "IAM_UNAUTHORIZED",
            RecommendationPermission::PlacementManage => "RECOMMENDATION_PLACEMENT_FORBIDDEN",
            RecommendationPermission::RankingManage => "RECOMMENDATION_RANKING_FORBIDDEN",
            RecommendationPermission::RebuildExecute => "RECOMMENDATION_REBUILD_FORBIDDEN",
        }
    }
}

pub fn is_allowed(role: &str, permission: RecommendationPermission) -> bool {
    role_has_permission(role, permission)
}

pub fn is_allowed_roles(roles: &[String], permission: RecommendationPermission) -> bool {
    roles
        .iter()
        .any(|role| role_has_permission(role.as_str(), permission))
}

pub fn first_matching_role(
    roles: &[String],
    permission: RecommendationPermission,
) -> Option<String> {
    roles
        .iter()
        .find(|role| role_has_permission(role.as_str(), permission))
        .cloned()
}

fn role_has_permission(role: &str, permission: RecommendationPermission) -> bool {
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
