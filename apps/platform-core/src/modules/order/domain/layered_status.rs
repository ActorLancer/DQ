#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredOrderStatus {
    pub delivery_status: String,
    pub acceptance_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
}

pub fn derive_layered_status(current_state: &str, payment_status: &str) -> LayeredOrderStatus {
    let delivery_status = match current_state {
        "created"
        | "contract_pending"
        | "contract_effective"
        | "buyer_locked"
        | "payment_failed_pending_resolution"
        | "payment_timeout_pending_compensation_cancel" => "pending_delivery",
        "seller_delivering" => "in_progress",
        "delivered" | "accepted" | "settled" => "delivered",
        "closed" => "closed",
        _ => "pending_delivery",
    };

    let acceptance_status = match current_state {
        "delivered" => "pending_acceptance",
        "accepted" | "settled" => "accepted",
        "closed" => "closed",
        _ => "not_started",
    };

    let settlement_status = match current_state {
        "settled" => "settled",
        "closed" => "closed",
        _ => {
            if payment_status == "paid" {
                "pending_settlement"
            } else {
                "not_started"
            }
        }
    };

    LayeredOrderStatus {
        delivery_status: delivery_status.to_string(),
        acceptance_status: acceptance_status.to_string(),
        settlement_status: settlement_status.to_string(),
        dispute_status: "none".to_string(),
    }
}

pub fn derive_closed_layered_status_by_reason(reason_code: &str) -> LayeredOrderStatus {
    if reason_code.starts_with("order_cancel_") {
        return LayeredOrderStatus {
            delivery_status: "canceled".to_string(),
            acceptance_status: "canceled".to_string(),
            settlement_status: "canceled".to_string(),
            dispute_status: "none".to_string(),
        };
    }
    derive_layered_status("closed", "paid")
}

#[cfg(test)]
mod tests {
    use super::{derive_closed_layered_status_by_reason, derive_layered_status};

    #[test]
    fn created_maps_to_pending_substates() {
        let status = derive_layered_status("created", "unpaid");
        assert_eq!(status.delivery_status, "pending_delivery");
        assert_eq!(status.acceptance_status, "not_started");
        assert_eq!(status.settlement_status, "not_started");
        assert_eq!(status.dispute_status, "none");
    }

    #[test]
    fn cancel_close_maps_to_canceled_substates() {
        let status =
            derive_closed_layered_status_by_reason("order_cancel_refund_required_after_lock");
        assert_eq!(status.delivery_status, "canceled");
        assert_eq!(status.acceptance_status, "canceled");
        assert_eq!(status.settlement_status, "canceled");
    }
}
