import type { Metadata } from "next";
import { IBM_Plex_Mono, IBM_Plex_Sans } from "next/font/google";
import type { ReactNode } from "react";

import { AuthPlaceholderDialog } from "@/components/portal/auth-placeholder-dialog";
import { IdentityStrip } from "@/components/portal/identity-strip";
import { PortalNavigation } from "@/components/portal/navigation";
import { readPortalSession, readPortalSessionPreview } from "@/lib/session";
import { AppProviders } from "@/providers/app-providers";

import "./globals.css";

const sans = IBM_Plex_Sans({
  subsets: ["latin"],
  variable: "--font-plex-sans",
  weight: ["400", "500", "600", "700"],
});

const mono = IBM_Plex_Mono({
  subsets: ["latin"],
  variable: "--font-plex-mono",
  weight: ["400", "500"],
});

export const metadata: Metadata = {
  title: {
    default: "DataB Portal",
    template: "%s | DataB Portal",
  },
  description: "V1 Core 门户前端基线与受控 API 接入边界。",
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  const session = await readPortalSession();
  const sessionPreview = readPortalSessionPreview(session);
  const sessionLabel =
    session.mode === "bearer"
      ? session.label || "Bearer Session"
      : session.mode === "local"
        ? `${session.loginId} / ${session.role}`
        : "Guest";

  return (
    <html lang="zh-CN">
      <body className={`${sans.variable} ${mono.variable} min-h-screen bg-[var(--page)] font-sans text-[var(--ink-strong)] antialiased`}>
        <AppProviders>
          <div className="mx-auto flex min-h-screen w-full max-w-[1680px] flex-col gap-4 px-4 py-4 lg:flex-row lg:px-6">
            <aside className="w-full shrink-0 lg:sticky lg:top-4 lg:w-[320px] lg:self-start">
              <PortalNavigation />
            </aside>
            <main className="flex-1 space-y-4">
              <div className="flex flex-col gap-3 xl:flex-row xl:items-center">
                <div className="flex-1">
                  <IdentityStrip
                    sessionLabel={sessionLabel}
                    sessionMode={session.mode}
                    initialSubject={sessionPreview}
                  />
                </div>
                <div className="flex shrink-0 justify-end">
                  <AuthPlaceholderDialog />
                </div>
              </div>
              {children}
            </main>
          </div>
        </AppProviders>
      </body>
    </html>
  );
}
