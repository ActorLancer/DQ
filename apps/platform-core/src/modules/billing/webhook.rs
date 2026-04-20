//! webhook 纯函数（签名验证、时间戳、状态映射）

use crate::modules::billing::models::PaymentWebhookRequest;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn now_utc_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn parse_webhook_timestamp_ms(
    header_timestamp: Option<&str>,
    payload: &PaymentWebhookRequest,
) -> Option<i64> {
    let parse = |raw: &str| raw.trim().parse::<i64>().ok().map(normalize_epoch_ms);
    header_timestamp
        .and_then(parse)
        .or(payload.occurred_at_ms.map(normalize_epoch_ms))
}

fn normalize_epoch_ms(value: i64) -> i64 {
    if value < 10_000_000_000 {
        value * 1000
    } else {
        value
    }
}

pub(crate) fn is_replay_window_valid(occurred_at_ms: i64) -> bool {
    let now_ms = now_utc_ms();
    let max_backward_ms = 15 * 60 * 1000;
    let max_forward_ms = 2 * 60 * 1000;
    occurred_at_ms >= now_ms - max_backward_ms && occurred_at_ms <= now_ms + max_forward_ms
}

pub fn verify_webhook_signature_placeholder(
    provider: &str,
    signature: Option<&str>,
    payload: &PaymentWebhookRequest,
) -> bool {
    if payload.provider_event_id.trim().is_empty() || payload.event_type.trim().is_empty() {
        return false;
    }
    let expected = std::env::var("MOCK_PAYMENT_WEBHOOK_SIGNATURE")
        .unwrap_or_else(|_| "mock-signature".to_string());
    if provider == "mock_payment" {
        return signature.map(|v| v.trim() == expected).unwrap_or(false);
    }
    signature.map(|v| !v.trim().is_empty()).unwrap_or(false)
}

pub(crate) fn map_webhook_target_status(
    event_type: &str,
    provider_status: Option<&str>,
) -> Option<&'static str> {
    let normalized_type = event_type.trim().to_ascii_lowercase();
    let normalized_status = provider_status.unwrap_or("").trim().to_ascii_lowercase();
    if normalized_type.contains("succeeded") || normalized_status == "succeeded" {
        return Some("succeeded");
    }
    if normalized_type.contains("failed") || normalized_status == "failed" {
        return Some("failed");
    }
    if normalized_type.contains("timeout") || normalized_status == "timeout" {
        return Some("expired");
    }
    None
}

pub(crate) fn payment_status_rank(status: &str) -> i32 {
    match status {
        "created" => 0,
        "pending" | "processing" | "locked" => 1,
        "failed" | "expired" | "canceled" => 2,
        "succeeded" => 3,
        _ => 0,
    }
}

pub(crate) fn map_webhook_transaction_shape(
    event_type: &str,
) -> Option<(&'static str, &'static str)> {
    let normalized_type = event_type.trim().to_ascii_lowercase();
    if normalized_type.starts_with("payment.") {
        return Some(("payin", "inbound"));
    }
    None
}
