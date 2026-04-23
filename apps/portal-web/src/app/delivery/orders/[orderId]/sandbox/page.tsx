import { PortalRoutePage } from "@/components/portal/route-page";

export default async function DeliverySandboxPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return <PortalRoutePage routeKey="delivery_sandbox" params={await params} />;
}
