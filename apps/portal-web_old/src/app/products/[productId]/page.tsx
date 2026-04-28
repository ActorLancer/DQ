import { ProductDetailShell } from "@/components/portal/product-detail-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function ProductDetailPage({
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
    <ProductDetailShell
      productId={resolvedParams.productId}
      sessionMode={session.mode}
      initialSubject={sessionPreview}
    />
  );
}
