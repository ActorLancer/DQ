import { BillingCenterShell } from "@/components/portal/billing-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function BillingPage() {
  const session = await readPortalSession();
  return (
    <BillingCenterShell
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
