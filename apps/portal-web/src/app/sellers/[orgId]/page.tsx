import { PortalRoutePage } from "@/components/portal/route-page";

export default async function SellerProfilePage({
  params,
}: {
  params: Promise<{ orgId: string }>;
}) {
  const resolvedParams = await params;
  return <PortalRoutePage routeKey="seller_profile" params={resolvedParams} />;
}
