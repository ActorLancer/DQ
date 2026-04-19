use crate::error::{Error, Result};
use crate::sqlx::postgres::build_query_with_params;
use async_trait::async_trait;
use sqlx::postgres::{PgArguments, PgPool, PgRow};
use sqlx::{Arguments, ColumnIndex, Postgres, Transaction as SqlxTransaction, Type};
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;

pub mod tls {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct NoTlsStream;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoTls;

#[derive(Debug, Clone, Copy, Default)]
pub struct Socket;

#[derive(Debug)]
pub struct Connection<S = Socket, T = tls::NoTlsStream> {
    _marker: PhantomData<(S, T)>,
}

impl<S, T> Default for Connection<S, T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<S, T> Future for Connection<S, T> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok(()))
    }
}

pub trait DbParam: Send + Sync {
    fn add_to_args(&self, args: &mut PgArguments) -> Result<()>;
}

macro_rules! impl_owned_db_param {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl DbParam for $ty {
                fn add_to_args(&self, args: &mut PgArguments) -> Result<()> {
                    args.add(self.clone())
                        .map_err(|err| Error::Bind(err.to_string()))
                }
            }
        )+
    };
}

impl_owned_db_param!(
    String,
    Option<String>,
    bool,
    Option<bool>,
    i16,
    Option<i16>,
    i32,
    Option<i32>,
    i64,
    Option<i64>,
    f32,
    Option<f32>,
    f64,
    Option<f64>,
    Vec<String>,
    Option<Vec<String>>,
    serde_json::Value,
    Option<serde_json::Value>,
    uuid::Uuid,
    Option<uuid::Uuid>,
);

impl DbParam for str {
    fn add_to_args(&self, args: &mut PgArguments) -> Result<()> {
        args.add(self.to_string())
            .map_err(|err| Error::Bind(err.to_string()))
    }
}

impl DbParam for &str {
    fn add_to_args(&self, args: &mut PgArguments) -> Result<()> {
        args.add((*self).to_string())
            .map_err(|err| Error::Bind(err.to_string()))
    }
}

impl DbParam for Option<&str> {
    fn add_to_args(&self, args: &mut PgArguments) -> Result<()> {
        args.add(self.map(str::to_string))
            .map_err(|err| Error::Bind(err.to_string()))
    }
}

impl DbParam for &[String] {
    fn add_to_args(&self, args: &mut PgArguments) -> Result<()> {
        args.add(self.to_vec())
            .map_err(|err| Error::Bind(err.to_string()))
    }
}

#[derive(Debug)]
pub struct Row {
    inner: PgRow,
}

impl Row {
    pub fn new(inner: PgRow) -> Self {
        Self { inner }
    }

    pub fn get<I, T>(&self, index: I) -> T
    where
        I: ColumnIndex<PgRow>,
        T: for<'r> sqlx::Decode<'r, Postgres> + Type<Postgres>,
    {
        sqlx::Row::get(&self.inner, index)
    }

    pub fn try_get<I, T>(&self, index: I) -> std::result::Result<T, sqlx::Error>
    where
        I: ColumnIndex<PgRow>,
        T: for<'r> sqlx::Decode<'r, Postgres> + Type<Postgres>,
    {
        sqlx::Row::try_get(&self.inner, index)
    }
}

impl From<PgRow> for Row {
    fn from(value: PgRow) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pool: Arc<PgPool>,
}

impl Client {
    pub fn from_pool(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn transaction(&self) -> Result<Transaction> {
        let tx = self.pool.begin().await?;
        Ok(Transaction {
            inner: Arc::new(Mutex::new(Some(tx))),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    inner: Arc<Mutex<Option<SqlxTransaction<'static, Postgres>>>>,
}

impl Transaction {
    pub async fn commit(self) -> Result<()> {
        let tx = self.take_inner().await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        let tx = self.take_inner().await?;
        tx.rollback().await?;
        Ok(())
    }

    async fn take_inner(&self) -> Result<SqlxTransaction<'static, Postgres>> {
        self.inner
            .lock()
            .await
            .take()
            .ok_or_else(|| Error::Backend("transaction already completed".to_string()))
    }
}

#[async_trait]
pub trait GenericClient: Send + Sync + Debug {
    async fn query(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Vec<Row>>;
    async fn query_opt(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Option<Row>>;
    async fn query_one(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Row>;
    async fn execute(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<u64>;
}

#[async_trait]
impl GenericClient for Client {
    async fn query(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Vec<Row>> {
        let query = build_query_with_params(sql, params)?;
        let rows = query.fetch_all(self.pool.as_ref()).await?;
        Ok(rows.into_iter().map(Row::from).collect())
    }

    async fn query_opt(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Option<Row>> {
        let query = build_query_with_params(sql, params)?;
        Ok(query
            .fetch_optional(self.pool.as_ref())
            .await?
            .map(Row::from))
    }

    async fn query_one(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Row> {
        let query = build_query_with_params(sql, params)?;
        Ok(query.fetch_one(self.pool.as_ref()).await?.into())
    }

    async fn execute(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<u64> {
        let query = build_query_with_params(sql, params)?;
        Ok(query.execute(self.pool.as_ref()).await?.rows_affected())
    }
}

#[async_trait]
impl GenericClient for Transaction {
    async fn query(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Vec<Row>> {
        let query = build_query_with_params(sql, params)?;
        let mut guard = self.inner.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| Error::Backend("transaction already completed".to_string()))?;
        let rows = query.fetch_all(&mut **tx).await?;
        Ok(rows.into_iter().map(Row::from).collect())
    }

    async fn query_opt(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Option<Row>> {
        let query = build_query_with_params(sql, params)?;
        let mut guard = self.inner.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| Error::Backend("transaction already completed".to_string()))?;
        Ok(query.fetch_optional(&mut **tx).await?.map(Row::from))
    }

    async fn query_one(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<Row> {
        let query = build_query_with_params(sql, params)?;
        let mut guard = self.inner.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| Error::Backend("transaction already completed".to_string()))?;
        Ok(query.fetch_one(&mut **tx).await?.into())
    }

    async fn execute(&self, sql: &str, params: &[&(dyn DbParam + Sync)]) -> Result<u64> {
        let query = build_query_with_params(sql, params)?;
        let mut guard = self.inner.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| Error::Backend("transaction already completed".to_string()))?;
        Ok(query.execute(&mut **tx).await?.rows_affected())
    }
}
