use crate::error::Result;
use crate::runtime::{Client, NoTls, connect};

pub fn test_database_url() -> String {
    std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("TEST_DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string())
}

pub async fn connect_test_client() -> Result<Client> {
    let (client, connection) = connect(&test_database_url(), NoTls).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok(client)
}
