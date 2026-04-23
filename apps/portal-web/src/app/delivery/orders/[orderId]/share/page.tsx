import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliverySharePage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return <PortalRoutePage routeKey="delivery_share" params={await params} />;
}
