import { HomeShell } from "@/components/portal/home-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function Home() {
  const session = await readPortalSession();
  const sessionPreview = readPortalSessionPreview(session);

  return <HomeShell sessionMode={session.mode} initialSubject={sessionPreview} />;
}
