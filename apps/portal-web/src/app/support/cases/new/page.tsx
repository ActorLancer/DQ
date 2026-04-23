import { DisputeWorkflowShell } from "@/components/portal/dispute-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function DisputeCreatePage() {
  const session = await readPortalSession();
  return (
    <DisputeWorkflowShell
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
