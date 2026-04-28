import Link from "next/link";

import { CommandPalette } from "./command-palette";

export function TopNav() {
  return (
    <header className="sticky top-0 z-40 border-b border-slate-200 bg-white">
      <div className="mx-auto flex h-14 max-w-[1680px] items-center justify-between px-4">
        <Link href="/" className="text-sm font-semibold tracking-wide text-slate-900">
          Datab Exchange Console
        </Link>
        <div className="flex items-center gap-3">
          <CommandPalette />
          <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs text-slate-600">
            Subject: tenant_developer · Role: buyer_operator · Tenant: 10000000...0102
          </div>
        </div>
      </div>
    </header>
  );
}
