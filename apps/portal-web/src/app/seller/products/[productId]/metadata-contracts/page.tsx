import { SellerProductWorkspaceShell } from "@/components/portal/seller-product-workspace-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function MetadataContractsPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  const sessionPreview = readPortalSessionPreview(session);

  return (
    <SellerProductWorkspaceShell
      initialSection="metadata"
      productId={resolvedParams.productId}
      sessionMode={session.mode}
      initialSubject={sessionPreview}
    />
  );
}
