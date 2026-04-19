#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentResultKind {
    Succeeded,
    Failed,
    TimedOut,
}

pub fn is_payment_mutable_order_status(status: &str) -> bool {
    matches!(status, "created" | "contract_effective")
}

pub fn derive_target_state(
    current_status: &str,
    result: PaymentResultKind,
) -> Option<(&'static str, &'static str, &'static str)> {
    if !is_payment_mutable_order_status(current_status) {
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
    fn created_order_can_enter_payment_failure_resolution() {
        let target = derive_target_state("created", PaymentResultKind::Failed);
        assert_eq!(
            target,
            Some((
                "payment_failed_pending_resolution",
                "failed",
                "payment_failed_pending_resolution"
            ))
        );
    }

    #[test]
    fn contract_pending_order_ignores_payment_callback() {
        let target = derive_target_state("contract_pending", PaymentResultKind::Succeeded);
        assert_eq!(target, None);
    }

    #[test]
    fn approval_pending_order_ignores_payment_callback() {
        let target = derive_target_state("approval_pending", PaymentResultKind::TimedOut);
        assert_eq!(target, None);
    }

    #[test]
    fn failure_does_not_rollback_after_delivery_progress() {
        let target = derive_target_state("seller_delivering", PaymentResultKind::Failed);
        assert_eq!(target, None);
    }

    #[test]
    fn paid_order_ignores_late_failure_callback() {
        let target = derive_target_state("buyer_locked", PaymentResultKind::Failed);
        assert_eq!(target, None);
    }

    #[test]
    fn sku_specific_fulfillment_state_ignores_payment_callback() {
        let target = derive_target_state("api_bound", PaymentResultKind::TimedOut);
        assert_eq!(target, None);
    }
}
