use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("bind parameter encode failed: {0}")]
    Bind(String),
    #[error("backend unavailable: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, Error>;
