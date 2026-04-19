use crate::error::Result;
use crate::runtime::transaction::{Client, Connection, NoTls, Socket, tls};
use crate::sqlx::postgres::shared_pool_with_defaults;

pub async fn connect(
    dsn: &str,
    _tls: NoTls,
) -> Result<(Client, Connection<Socket, tls::NoTlsStream>)> {
    let pool = shared_pool_with_defaults(dsn).await?;
    Ok((Client::from_pool(pool), Connection::default()))
}
