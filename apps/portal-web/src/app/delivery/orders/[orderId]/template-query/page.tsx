import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliveryTemplateQueryPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="delivery_template_query"
      params={await params}
    />
  );
}
