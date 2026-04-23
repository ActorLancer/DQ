import { PortalRoutePage } from "@/components/portal/route-page";

export default async function SellerProductEditPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="seller_product_edit"
      params={await params}
    />
  );
}
