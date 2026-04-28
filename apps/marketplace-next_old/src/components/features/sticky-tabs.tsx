"use client";

import { motion } from "motion/react";
import { startTransition } from "react";

import { cn } from "@/lib/utils";

const tabs = ["overview", "schema", "sample", "pricing", "docs", "reviews"] as const;

export function StickyTabs({
  value,
  onChange,
}: {
  value: (typeof tabs)[number];
  onChange: (next: (typeof tabs)[number]) => void;
}) {
  return (
    <div className="sticky top-0 z-20 mb-3 rounded-xl border border-slate-200 bg-white/95 p-1 shadow-sm backdrop-blur">
      <div className="flex gap-1">
        {tabs.map((tab) => (
          <button
            key={tab}
            type="button"
            onClick={() => {
              startTransition(() => onChange(tab));
            }}
            className={cn(
              "relative rounded-lg px-3 py-1.5 text-xs font-medium transition",
              value === tab
                ? "bg-slate-900 text-white"
                : "text-slate-600 hover:bg-slate-100 hover:text-slate-900",
            )}
          >
            {value === tab ? (
              <motion.span
                layoutId="detail-tab-pill"
                className="absolute inset-0 -z-10 rounded-lg bg-slate-900"
                transition={{ type: "spring", stiffness: 340, damping: 30 }}
              />
            ) : null}
            {tab.toUpperCase()}
          </button>
        ))}
      </div>
    </div>
  );
}
