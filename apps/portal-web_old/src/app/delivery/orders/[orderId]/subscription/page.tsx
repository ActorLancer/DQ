import { DeliveryWorkflowShell } from "@/components/portal/delivery-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function DeliverySubscriptionPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  return (
    <DeliveryWorkflowShell
      kind="subscription"
      orderId={resolvedParams.orderId}
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
