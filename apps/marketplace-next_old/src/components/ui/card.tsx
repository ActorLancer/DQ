import { ComponentPropsWithoutRef } from "react";

import { cn } from "@/lib/utils";

export function Card({ className, ...props }: ComponentPropsWithoutRef<"div">) {
  return (
    <div
      className={cn(
        "rounded-2xl border border-slate-200 bg-white/95 shadow-[0_1px_2px_rgba(15,23,42,0.04)] backdrop-blur",
        className,
      )}
      {...props}
    />
  );
}
