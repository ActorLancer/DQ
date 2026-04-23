import { PortalRoutePage } from "@/components/portal/route-page";

export default async function SellerQuerySurfacePage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="seller_query_surface"
      params={await params}
    />
  );
}
