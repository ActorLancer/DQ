import { PortalRoutePage } from "@/components/portal/route-page";

export default async function PaymentLockPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="order_payment_lock"
      params={await params}
    />
  );
}
