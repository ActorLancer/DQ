import { OrderCreateShell } from "@/components/portal/order-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function OrderCreatePage({
  searchParams,
}: {
  searchParams: Promise<{
    productId?: string;
    product_id?: string;
    scenario?: string;
    scenario_code?: string;
    skuId?: string;
    sku_id?: string;
  }>;
}) {
  const [resolvedSearchParams, session] = await Promise.all([
    searchParams,
    readPortalSession(),
  ]);
  const sessionPreview = readPortalSessionPreview(session);

  return (
    <OrderCreateShell
      productId={resolvedSearchParams.productId ?? resolvedSearchParams.product_id}
      initialScenario={
        resolvedSearchParams.scenario ?? resolvedSearchParams.scenario_code
      }
      initialSkuId={resolvedSearchParams.skuId ?? resolvedSearchParams.sku_id}
      sessionMode={session.mode}
      initialSubject={sessionPreview}
    />
  );
}
