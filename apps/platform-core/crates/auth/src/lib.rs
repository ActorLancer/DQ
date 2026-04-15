use axum::http::{HeaderMap, HeaderValue};
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
}
