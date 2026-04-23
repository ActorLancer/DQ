import { PortalRoutePage } from "@/components/portal/route-page";

export default async function OrderDetailPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return <PortalRoutePage routeKey="order_detail" params={await params} />;
}
