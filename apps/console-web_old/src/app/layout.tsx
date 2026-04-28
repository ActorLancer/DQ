import type { Metadata } from "next";
import { IBM_Plex_Mono, Space_Grotesk } from "next/font/google";
import type { ReactNode } from "react";

import { AuthSessionDialog } from "@/components/console/auth-session-dialog";
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
          <div className="mx-auto flex min-h-screen w-full max-w-[1760px] flex-col gap-5 px-4 py-4 lg:flex-row lg:px-6 lg:py-6">
            <aside className="w-full shrink-0 lg:sticky lg:top-5 lg:w-[360px] lg:self-start">
              <ConsoleNavigation />
            </aside>
            <main className="min-w-0 flex-1 space-y-5">
              <section className="rounded-[30px] border border-[var(--surface-ring)] bg-[var(--surface-strong)] px-5 py-4 shadow-[var(--surface-shadow)] ring-1 ring-white/55">
                <div className="flex flex-col gap-3 xl:flex-row xl:items-center">
                  <div className="flex-1">
                    <div className="inline-flex rounded-full bg-[var(--accent-soft)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-[var(--accent-strong)]">
                      DataB Control Plane
                    </div>
                    <p className="mt-2 text-sm text-[var(--ink-soft)]">
                      审核、审计、Ops、通知与开发者联查控制面。侧重高密度信息组织与可追溯操作反馈。
                    </p>
                  </div>
                  <div className="text-xs text-[var(--ink-subtle)]">
                    Governance-first UX · Traceability · Step-up Ready
                  </div>
                </div>
              </section>
              <div className="flex flex-col gap-3 xl:flex-row xl:items-center">
                <div className="flex-1">
                  <IdentityStrip sessionLabel={sessionLabel} />
                </div>
                <div className="flex shrink-0 justify-end">
                  <AuthSessionDialog />
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
