import { PortalRoutePage } from "@/components/portal/route-page";

export default async function SellerSkuConfigPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="seller_sku_config"
      params={await params}
    />
  );
}
