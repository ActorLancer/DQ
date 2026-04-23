import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliverySubscriptionPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="delivery_subscription"
      params={await params}
    />
  );
}
