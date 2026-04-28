"use client";

import { motion } from "motion/react";
import Link from "next/link";
import { usePathname } from "next/navigation";

import { navItems } from "./nav-config";

export function SideNav() {
  const pathname = usePathname();

  return (
    <aside className="w-64 shrink-0 rounded-xl panel p-2">
      <div className="mb-2 rounded-lg panel-muted p-2">
        <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-slate-500">Workspace</p>
        <p className="mt-1 text-xs text-slate-700">Data Catalog / Market / Operations</p>
      </div>
      <nav className="space-y-1">
        {navItems.map((item, idx) => {
          const active = pathname === item.href || pathname.startsWith(`${item.href}/`);
          const Icon = item.icon;
          return (
            <motion.div
              key={item.href}
              initial={{ opacity: 0, x: -6 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.2, delay: idx * 0.03 }}
            >
              <Link
                href={item.href}
                className={`flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition ${
                  active
                    ? "bg-slate-900 text-white shadow-sm"
                    : "text-slate-600 hover:bg-slate-100 hover:text-slate-900"
                }`}
              >
                <Icon className="size-3.5" />
                {item.label}
              </Link>
            </motion.div>
          );
        })}
      </nav>
    </aside>
  );
}
