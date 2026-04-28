import { SellerShareModesShell } from "@/components/portal/advanced-route-shells";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function SellerShareModesPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  return (
    <SellerShareModesShell
      productId={resolvedParams.productId}
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
