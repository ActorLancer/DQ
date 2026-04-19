use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Builder, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use axum::Json;
use axum::http::StatusCode;
use kernel::{ErrorCode, ErrorResponse};

#[derive(Debug, Clone)]
pub struct FetchedObjectPayload {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

pub async fn put_object_bytes(
    bucket_name: &str,
    object_key: &str,
    bytes: Vec<u8>,
    content_type: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let client = build_minio_client();
    let mut request = client
        .put_object()
        .bucket(bucket_name)
        .key(object_key)
        .body(ByteStream::from(bytes));
    if let Some(content_type) = content_type.filter(|value| !value.trim().is_empty()) {
        request = request.content_type(content_type);
    }
    request.send().await.map_err(map_object_store_error)?;
    Ok(())
}

pub async fn delete_object(
    bucket_name: &str,
    object_key: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let client = build_minio_client();
    client
        .delete_object()
        .bucket(bucket_name)
        .key(object_key)
        .send()
        .await
        .map_err(map_object_store_error)?;
    Ok(())
}

pub async fn fetch_object_bytes(
    bucket_name: &str,
    object_key: &str,
) -> Result<FetchedObjectPayload, (StatusCode, Json<ErrorResponse>)> {
    let client = build_minio_client();
    let output = client
        .get_object()
        .bucket(bucket_name)
        .key(object_key)
        .send()
        .await
        .map_err(map_object_store_error)?;
    let content_type = output.content_type().map(str::to_string);
    let body = output
        .body
        .collect()
        .await
        .map_err(map_object_store_error)?;
    Ok(FetchedObjectPayload {
        bytes: body.into_bytes().to_vec(),
        content_type,
    })
}

fn build_minio_client() -> Client {
    let endpoint =
        std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9000".to_string());
    let access_key = std::env::var("MINIO_ROOT_USER").unwrap_or_else(|_| "datab".to_string());
    let secret_key =
        std::env::var("MINIO_ROOT_PASSWORD").unwrap_or_else(|_| "datab_local_pass".to_string());
    let region = std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string());

    let config = Builder::new()
        .behavior_version_latest()
        .endpoint_url(endpoint)
        .credentials_provider(Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "platform-core-delivery",
        ))
        .region(Region::new(region))
        .force_path_style(true)
        .build();
    Client::from_conf(config)
}

fn map_object_store_error(err: impl std::fmt::Display) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_GATEWAY,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("delivery object fetch failed: {err}"),
            request_id: None,
        }),
    )
}
