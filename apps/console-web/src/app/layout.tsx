import type { Metadata } from "next";
import { IBM_Plex_Mono, Space_Grotesk } from "next/font/google";
import type { ReactNode } from "react";

import { AuthPlaceholderDialog } from "@/components/console/auth-placeholder-dialog";
import { ConsoleNavigation } from "@/components/console/navigation";
import { IdentityStrip } from "@/components/console/identity-strip";
import { AppProviders } from "@/providers/app-providers";
import { readConsoleSession } from "@/lib/session";

import "./globals.css";

const sans = Space_Grotesk({
  subsets: ["latin"],
  variable: "--font-space-grotesk",
  weight: ["400", "500", "600", "700"],
});

const mono = IBM_Plex_Mono({
  subsets: ["latin"],
  variable: "--font-plex-mono",
  weight: ["400", "500"],
});

export const metadata: Metadata = {
  title: {
    default: "DataB Console",
    template: "%s | DataB Console",
  },
  description: "V1 Core 控制台前端基线与受控 API 接入边界。",
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  const session = await readConsoleSession();
  const sessionLabel =
    session.mode === "bearer"
      ? session.label || "Control Plane Bearer"
      : session.mode === "local"
        ? `${session.loginId} / ${session.role}`
        : "Guest";

  return (
    <html lang="zh-CN" className="overflow-x-hidden">
      <body
        className={`${sans.variable} ${mono.variable} min-h-screen overflow-x-hidden bg-[var(--page)] font-sans text-[var(--ink-strong)] antialiased`}
      >
        <AppProviders>
          <div className="mx-auto flex min-h-screen w-full max-w-[1720px] flex-col gap-4 px-4 py-4 lg:flex-row lg:px-6">
            <aside className="w-full shrink-0 lg:sticky lg:top-4 lg:w-[340px] lg:self-start">
              <ConsoleNavigation />
            </aside>
            <main className="min-w-0 flex-1 space-y-4">
              <div className="flex flex-col gap-3 xl:flex-row xl:items-center">
                <div className="flex-1">
                  <IdentityStrip sessionLabel={sessionLabel} />
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
