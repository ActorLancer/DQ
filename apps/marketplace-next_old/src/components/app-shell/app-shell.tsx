import { ReactNode } from "react";

import { SideNav } from "./side-nav";
import { TopNav } from "./top-nav";

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-screen bg-[var(--app-bg)]">
      <TopNav />
      <div className="mx-auto flex max-w-[1680px] gap-3 px-4 py-3">
        <SideNav />
        <main className="h-[calc(100vh-5.25rem)] flex-1 overflow-hidden rounded-xl panel p-3">
          {children}
        </main>
      </div>
    </div>
  );
}
