import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliveryAcceptancePage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="delivery_acceptance"
      params={await params}
    />
  );
}
