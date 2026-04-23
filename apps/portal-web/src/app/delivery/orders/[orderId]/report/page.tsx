import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliveryReportPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return <PortalRoutePage routeKey="delivery_report" params={await params} />;
}
