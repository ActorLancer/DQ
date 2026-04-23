import { AssetRawIngestShell } from "@/components/portal/advanced-route-shells";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function RawIngestPage({
  params,
}: {
  params: Promise<{ assetId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  return (
    <AssetRawIngestShell
      assetId={resolvedParams.assetId}
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
