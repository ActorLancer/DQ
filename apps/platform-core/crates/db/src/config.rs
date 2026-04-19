#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    pub dsn: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct AppDbConfig {
    pub dsn: String,
    pub max_connections: u32,
}

impl From<DbPoolConfig> for AppDbConfig {
    fn from(value: DbPoolConfig) -> Self {
        Self {
            dsn: value.dsn,
            max_connections: value.max_connections,
        }
    }
}
