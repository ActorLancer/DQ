import type { Metadata } from "next";
import { ReactNode } from "react";

import { AppShell } from "@/components/app-shell/app-shell";

import "./globals.css";

export const metadata: Metadata = {
  title: "Datab Exchange Next",
  description: "New migration frontend for data exchange experience",
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="zh-CN">
      <body>
        <AppShell>{children}</AppShell>
      </body>
    </html>
  );
}
