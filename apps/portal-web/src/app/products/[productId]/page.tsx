import { PortalRoutePage } from "@/components/portal/route-page";

export default async function ProductDetailPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  const resolvedParams = await params;
  return <PortalRoutePage routeKey="product_detail" params={resolvedParams} />;
}
