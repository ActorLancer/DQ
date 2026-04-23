import { PortalRoutePage } from "@/components/portal/route-page";

export default async function MetadataContractsPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="seller_metadata_contract"
      params={await params}
    />
  );
}
