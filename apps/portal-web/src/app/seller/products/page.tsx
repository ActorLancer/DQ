import { SellerProductWorkspaceShell } from "@/components/portal/seller-product-workspace-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function SellerProductsPage() {
  const session = await readPortalSession();
  const sessionPreview = readPortalSessionPreview(session);

  return (
    <SellerProductWorkspaceShell
      initialSection="center"
      sessionMode={session.mode}
      initialSubject={sessionPreview}
    />
  );
}
