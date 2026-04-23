import { PortalRoutePage } from "@/components/portal/route-page";

export default async function SellerTemplatePage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="seller_template_bind"
      params={await params}
    />
  );
}
