use crate::config::AppDbConfig;
use crate::dialect::DatabaseDialect;
use crate::error::{Error, Result};
use crate::runtime::transaction::Client;
use crate::sqlx as db_sqlx;
use kernel::{AppError, AppResult};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PostgresDbRuntime {
    pub sqlx: Arc<PgPool>,
    pub orm: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct MySqlDbRuntime {
    pub dsn: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub enum AppDb {
    Postgres(PostgresDbRuntime),
    Mysql(MySqlDbRuntime),
}

#[derive(Debug, Clone)]
pub struct RepositoryBackendRegistry {
    dialect: DatabaseDialect,
}

impl RepositoryBackendRegistry {
    pub fn from_db(db: &AppDb) -> Self {
        Self {
            dialect: db.dialect(),
        }
    }

    pub fn dialect(&self) -> DatabaseDialect {
        self.dialect
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self.dialect, DatabaseDialect::Postgres)
    }

    pub fn is_mysql_reserved(&self) -> bool {
        matches!(self.dialect, DatabaseDialect::Mysql)
    }
}

impl AppDb {
    pub async fn connect(cfg: AppDbConfig) -> AppResult<Self> {
        match DatabaseDialect::detect(&cfg.dsn)? {
            DatabaseDialect::Postgres => {
                let sqlx = db_sqlx::postgres::shared_pool(&cfg)
                    .await
                    .map_err(to_app_error("postgres sqlx pool init failed"))?;
                let mut options = ConnectOptions::new(cfg.dsn.clone());
                options.max_connections(cfg.max_connections);
                let orm = Database::connect(options).await.map_err(|err| {
                    AppError::Config(format!("postgres sea-orm init failed: {err}"))
                })?;
                Ok(Self::Postgres(PostgresDbRuntime { sqlx, orm }))
            }
            DatabaseDialect::Mysql => Ok(Self::Mysql(MySqlDbRuntime {
                dsn: cfg.dsn,
                max_connections: cfg.max_connections,
            })),
        }
    }

    pub fn dialect(&self) -> DatabaseDialect {
        match self {
            Self::Postgres(_) => DatabaseDialect::Postgres,
            Self::Mysql(_) => DatabaseDialect::Mysql,
        }
    }

    pub fn postgres(&self) -> AppResult<&PostgresDbRuntime> {
        match self {
            Self::Postgres(runtime) => Ok(runtime),
            Self::Mysql(_) => Err(AppError::Config(
                "mysql runtime seam is reserved for future expansion".to_string(),
            )),
        }
    }

    pub fn orm(&self) -> AppResult<DatabaseConnection> {
        Ok(self.postgres()?.orm.clone())
    }

    pub fn client(&self) -> Result<Client> {
        match self {
            Self::Postgres(runtime) => Ok(Client::from_pool(runtime.sqlx.clone())),
            Self::Mysql(_) => Err(Error::Backend(
                "mysql runtime seam is reserved for future expansion".to_string(),
            )),
        }
    }

    pub fn registry(&self) -> RepositoryBackendRegistry {
        RepositoryBackendRegistry::from_db(self)
    }
}

fn to_app_error(context: &'static str) -> impl Fn(Error) -> AppError {
    move |err| AppError::Config(format!("{context}: {err}"))
}
