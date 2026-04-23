import { BillingRefundCompensationShell } from "@/components/portal/billing-workflow-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function BillingRefundsPage() {
  const session = await readPortalSession();
  return (
    <BillingRefundCompensationShell
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
