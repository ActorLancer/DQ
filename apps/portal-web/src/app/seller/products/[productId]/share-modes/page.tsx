import { PortalRoutePage } from "@/components/portal/route-page";

export default async function SellerShareModesPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="seller_share_modes"
      params={await params}
    />
  );
}
