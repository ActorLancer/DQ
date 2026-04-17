use axum::http::{HeaderMap, HeaderValue};
use base64::Engine;
use kernel::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSubject {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
}

pub trait JwtParser: Send + Sync {
    fn parse_subject(&self, token: &str) -> AppResult<SessionSubject>;
}

#[derive(Debug, Default, Clone)]
pub struct MockJwtParser;

impl JwtParser for MockJwtParser {
    fn parse_subject(&self, token: &str) -> AppResult<SessionSubject> {
        if token.trim().is_empty() {
            return Err(AppError::Config("jwt token is empty".to_string()));
        }
        Ok(SessionSubject {
            user_id: "mock-user".to_string(),
            tenant_id: "mock-tenant".to_string(),
            roles: vec!["tenant_admin".to_string()],
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct KeycloakClaimsJwtParser;

impl JwtParser for KeycloakClaimsJwtParser {
    fn parse_subject(&self, token: &str) -> AppResult<SessionSubject> {
        if token.trim().is_empty() {
            return Err(AppError::Config("jwt token is empty".to_string()));
        }
        let mut segments = token.split('.');
        let _header = segments
            .next()
            .ok_or_else(|| AppError::Config("invalid jwt token format".to_string()))?;
        let payload = segments
            .next()
            .ok_or_else(|| AppError::Config("invalid jwt token format".to_string()))?;
        let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|err| AppError::Config(format!("jwt payload decode failed: {err}")))?;
        let claims: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|err| AppError::Config(format!("jwt claims parse failed: {err}")))?;
        let user_id = claims
            .get("sub")
            .and_then(|v| v.as_str())
            .or_else(|| claims.get("user_id").and_then(|v| v.as_str()))
            .ok_or_else(|| AppError::Config("jwt claims missing subject".to_string()))?
            .to_string();
        let tenant_id = claims
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .or_else(|| claims.get("org_id").and_then(|v| v.as_str()))
            .or_else(|| claims.get("azp").and_then(|v| v.as_str()))
            .unwrap_or("unknown-tenant")
            .to_string();
        let mut roles: Vec<String> = claims
            .get("realm_access")
            .and_then(|v| v.get("roles"))
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if roles.is_empty() {
            roles = claims
                .get("roles")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|v| v.as_str().map(ToString::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
        }
        if roles.is_empty() {
            roles.push("tenant_admin".to_string());
        }
        Ok(SessionSubject {
            user_id,
            tenant_id,
            roles,
        })
    }
}

pub trait PermissionChecker: Send + Sync {
    fn can(&self, subject: &SessionSubject, permission: &str) -> bool;
}

#[derive(Debug, Default, Clone)]
pub struct RolePermissionChecker;

impl PermissionChecker for RolePermissionChecker {
    fn can(&self, subject: &SessionSubject, permission: &str) -> bool {
        if subject.roles.iter().any(|r| r == "tenant_admin") {
            return true;
        }
        subject.roles.iter().any(|r| r == permission)
    }
}

pub trait StepUpGateway: Send + Sync {
    fn verify_step_up(&self, _subject: &SessionSubject) -> AppResult<()>;
}

#[derive(Debug, Default, Clone)]
pub struct NoopStepUpGateway;

impl StepUpGateway for NoopStepUpGateway {
    fn verify_step_up(&self, _subject: &SessionSubject) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationRequest {
    pub permission: String,
    pub require_step_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reason: Option<String>,
}

pub trait AuthorizationFacade: Send + Sync {
    fn resolve_subject(&self, headers: &HeaderMap) -> AppResult<SessionSubject>;
    fn evaluate(
        &self,
        subject: &SessionSubject,
        request: &AuthorizationRequest,
    ) -> AppResult<AuthorizationDecision>;
}

pub struct UnifiedAuthorizationFacade {
    pub jwt_parser: Box<dyn JwtParser>,
    pub permission_checker: Box<dyn PermissionChecker>,
    pub step_up_gateway: Box<dyn StepUpGateway>,
}

impl UnifiedAuthorizationFacade {
    pub fn new(
        jwt_parser: Box<dyn JwtParser>,
        permission_checker: Box<dyn PermissionChecker>,
        step_up_gateway: Box<dyn StepUpGateway>,
    ) -> Self {
        Self {
            jwt_parser,
            permission_checker,
            step_up_gateway,
        }
    }
}

impl AuthorizationFacade for UnifiedAuthorizationFacade {
    fn resolve_subject(&self, headers: &HeaderMap) -> AppResult<SessionSubject> {
        let token = extract_bearer(headers).ok_or_else(|| {
            AppError::Config("missing bearer token for authorization facade".to_string())
        })?;
        self.jwt_parser.parse_subject(&token)
    }

    fn evaluate(
        &self,
        subject: &SessionSubject,
        request: &AuthorizationRequest,
    ) -> AppResult<AuthorizationDecision> {
        if request.require_step_up {
            self.step_up_gateway.verify_step_up(subject)?;
        }

        let allowed = self.permission_checker.can(subject, &request.permission);
        Ok(AuthorizationDecision {
            allowed,
            reason: if allowed {
                None
            } else {
                Some(format!("permission denied: {}", request.permission))
            },
        })
    }
}

pub fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get(axum::http::header::AUTHORIZATION)?;
    parse_bearer_header(auth)
}

fn parse_bearer_header(value: &HeaderValue) -> Option<String> {
    let raw = value.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ")?;
    Some(token.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc.def.ghi"),
        );
        assert_eq!(extract_bearer(&headers), Some("abc.def.ghi".to_string()));
    }

    #[test]
    fn unified_authorization_facade_resolves_and_evaluates() {
        let facade = UnifiedAuthorizationFacade::new(
            Box::new(MockJwtParser),
            Box::new(RolePermissionChecker),
            Box::new(NoopStepUpGateway),
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer token.mock"),
        );
        let subject = facade.resolve_subject(&headers).expect("subject");
        let decision = facade
            .evaluate(
                &subject,
                &AuthorizationRequest {
                    permission: "order.create".to_string(),
                    require_step_up: true,
                },
            )
            .expect("decision");
        assert!(decision.allowed);
        assert_eq!(decision.reason, None);
    }

    #[test]
    fn unified_authorization_facade_denies_without_permission() {
        let facade = UnifiedAuthorizationFacade::new(
            Box::new(MockJwtParser),
            Box::new(RolePermissionChecker),
            Box::new(NoopStepUpGateway),
        );
        let subject = SessionSubject {
            user_id: "u-1".to_string(),
            tenant_id: "t-1".to_string(),
            roles: vec!["viewer".to_string()],
        };
        let decision = facade
            .evaluate(
                &subject,
                &AuthorizationRequest {
                    permission: "billing.settle".to_string(),
                    require_step_up: false,
                },
            )
            .expect("decision");
        assert!(!decision.allowed);
        assert_eq!(
            decision.reason.as_deref(),
            Some("permission denied: billing.settle")
        );
    }

    #[test]
    fn keycloak_claims_parser_extracts_subject_and_roles() {
        let parser = KeycloakClaimsJwtParser;
        let payload = serde_json::json!({
            "sub": "u-123",
            "tenant_id": "t-123",
            "realm_access": {"roles": ["tenant_admin", "auditor"]}
        });
        let payload_enc =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
        let token = format!("aaa.{payload_enc}.bbb");
        let subject = parser.parse_subject(&token).expect("subject parsed");
        assert_eq!(subject.user_id, "u-123");
        assert_eq!(subject.tenant_id, "t-123");
        assert!(subject.roles.iter().any(|r| r == "tenant_admin"));
    }
}
