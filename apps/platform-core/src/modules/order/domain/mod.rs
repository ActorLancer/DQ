#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentResultKind {
    Succeeded,
    Failed,
    TimedOut,
}

pub fn status_rank(status: &str) -> i32 {
    match status {
        "created" => 0,
        "quoted" => 1,
        "approval_pending" => 2,
        "contract_pending" => 3,
        "contract_effective" => 4,
        "buyer_locked" => 5,
        "seller_delivering" => 6,
        "delivered" => 7,
        "accepted" => 8,
        "settled" => 9,
        "closed" => 10,
        _ => 0,
    }
}

pub fn derive_target_state(
    current_status: &str,
    result: PaymentResultKind,
) -> Option<(&'static str, &'static str, &'static str)> {
    let current_rank = status_rank(current_status);
    let buyer_locked_rank = status_rank("buyer_locked");
    if current_rank > buyer_locked_rank {
        return None;
    }
    match result {
        PaymentResultKind::Succeeded => {
            Some(("buyer_locked", "paid", "payment_succeeded_to_buyer_locked"))
        }
        PaymentResultKind::Failed => Some((
            "payment_failed_pending_resolution",
            "failed",
            "payment_failed_pending_resolution",
        )),
        PaymentResultKind::TimedOut => Some((
            "payment_timeout_pending_compensation_cancel",
            "expired",
            "payment_timeout_pending_compensation_cancel",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_moves_pre_delivery_order_to_buyer_locked() {
        let target = derive_target_state("contract_effective", PaymentResultKind::Succeeded);
        assert_eq!(
            target,
            Some(("buyer_locked", "paid", "payment_succeeded_to_buyer_locked"))
        );
    }

    #[test]
    fn failure_does_not_rollback_after_delivery_progress() {
        let target = derive_target_state("seller_delivering", PaymentResultKind::Failed);
        assert_eq!(target, None);
    }
}
