import { SearchShell } from "@/components/portal/search-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";

export default async function SearchPage() {
  const session = await readPortalSession();
  const sessionPreview = readPortalSessionPreview(session);

  return <SearchShell sessionMode={session.mode} initialSubject={sessionPreview} />;
}
