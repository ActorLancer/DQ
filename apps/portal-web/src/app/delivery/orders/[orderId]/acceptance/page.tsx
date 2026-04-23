import { AcceptanceWorkflowShell } from "@/components/portal/acceptance-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function DeliveryAcceptancePage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  return (
    <AcceptanceWorkflowShell
      orderId={resolvedParams.orderId}
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
