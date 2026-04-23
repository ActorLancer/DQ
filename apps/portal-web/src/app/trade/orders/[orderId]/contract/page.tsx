import { OrderContractConfirmShell } from "@/components/portal/advanced-route-shells";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function ContractConfirmPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  return (
    <OrderContractConfirmShell
      orderId={resolvedParams.orderId}
      sessionMode={session.mode}
      initialSubject={readPortalSessionPreview(session)}
    />
  );
}
