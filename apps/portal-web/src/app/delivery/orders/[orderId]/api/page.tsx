import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliveryApiPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return <PortalRoutePage routeKey="delivery_api" params={await params} />;
}
