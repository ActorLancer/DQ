import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliveryQueryRunsPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="delivery_query_runs"
      params={await params}
    />
  );
}
