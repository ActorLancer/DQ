import { notFound } from "next/navigation";

import { StandardDemoShell } from "@/components/portal/standard-demo-shell";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";
import {
  findStandardDemoGuide,
  standardDemoGuides,
  type StandardScenarioCode,
} from "@/lib/standard-demo";

export function generateStaticParams() {
  return standardDemoGuides.map((guide) => ({
    scenarioCode: guide.scenarioCode,
  }));
}

export default async function StandardDemoPage({
  params,
}: {
  params: Promise<{ scenarioCode: string }>;
}) {
  const [{ scenarioCode }, session] = await Promise.all([
    params,
    readPortalSession(),
  ]);
  const guide = findStandardDemoGuide(scenarioCode);

  if (!guide) {
    notFound();
  }

  const sessionPreview = readPortalSessionPreview(session);

  return (
    <StandardDemoShell
      scenarioCode={guide.scenarioCode as StandardScenarioCode}
      sessionMode={session.mode}
      initialSubject={sessionPreview}
    />
  );
}
