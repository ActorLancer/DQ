use kernel::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseDialect {
    Postgres,
    Mysql,
}

impl DatabaseDialect {
    pub fn detect(dsn: &str) -> AppResult<Self> {
        let lowered = dsn.trim().to_ascii_lowercase();
        if lowered.starts_with("postgres://") || lowered.starts_with("postgresql://") {
            return Ok(Self::Postgres);
        }
        if lowered.starts_with("mysql://") {
            return Ok(Self::Mysql);
        }
        Err(AppError::Config(format!(
            "unsupported database dialect for dsn: {dsn}"
        )))
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
        }
    }
}
