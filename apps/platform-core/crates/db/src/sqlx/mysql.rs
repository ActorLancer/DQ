use crate::config::AppDbConfig;

#[derive(Debug, Clone)]
pub struct MySqlReservedRuntime {
    pub dsn: String,
    pub max_connections: u32,
}

impl From<&AppDbConfig> for MySqlReservedRuntime {
    fn from(value: &AppDbConfig) -> Self {
        Self {
            dsn: value.dsn.clone(),
            max_connections: value.max_connections,
        }
    }
}
