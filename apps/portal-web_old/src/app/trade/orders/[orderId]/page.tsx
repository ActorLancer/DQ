import { OrderDetailShell } from "@/components/portal/order-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function OrderDetailPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  const sessionPreview = readPortalSessionPreview(session);

  return (
    <OrderDetailShell
      orderId={resolvedParams.orderId}
      sessionMode={session.mode}
      initialSubject={sessionPreview}
    />
  );
}
