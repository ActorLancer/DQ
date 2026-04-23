import { PortalRoutePage } from "@/components/portal/route-page";

export default async function RawIngestPage({
  params,
}: {
  params: Promise<{ assetId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="asset_raw_ingest_center"
      params={await params}
    />
  );
}
