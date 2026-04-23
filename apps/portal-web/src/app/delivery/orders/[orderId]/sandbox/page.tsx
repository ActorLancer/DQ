import { DeliveryWorkflowShell } from "@/components/portal/delivery-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function DeliverySandboxPage({
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
      kind="sandbox"
      orderId={resolvedParams.orderId}
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
