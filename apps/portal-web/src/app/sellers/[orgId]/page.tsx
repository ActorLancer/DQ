import { SellerProfileShell } from "@/components/portal/seller-profile-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function SellerProfilePage({
  params,
}: {
  params: Promise<{ orgId: string }>;
}) {
  const [resolvedParams, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  const sessionPreview = readPortalSessionPreview(session);

  return (
    <SellerProfileShell
      orgId={resolvedParams.orgId}
      sessionMode={session.mode}
      initialSubject={sessionPreview}
    />
  );
}
