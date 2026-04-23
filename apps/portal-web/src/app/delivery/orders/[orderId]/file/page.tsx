import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliveryFilePage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return <PortalRoutePage routeKey="delivery_file" params={await params} />;
}
