use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::net::SocketAddr;
use tracing::info;

mod app;
mod modules;
mod shared;

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "platform-core",
        status: "ok",
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    let app = Router::new().route("/healthz", get(healthz));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    info!("platform-core listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind platform-core listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("serve platform-core");
}

