import { HTMLAttributes } from "react";

import { cn } from "@/lib/utils";

export function Card({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "group/card rounded-[30px] border border-[var(--surface-ring)] bg-[var(--surface)] p-5 shadow-[var(--surface-shadow)] backdrop-blur-xl ring-1 ring-white/45 transition duration-300 hover:border-[color:var(--accent-soft)] hover:shadow-[0_30px_88px_rgba(48,24,12,0.2)]",
        className,
      )}
      {...props}
    />
  );
}

export function CardTitle({
  className,
  ...props
}: HTMLAttributes<HTMLHeadingElement>) {
  return (
    <h3
      className={cn("text-lg font-semibold tracking-tight text-[var(--ink-strong)] md:text-[1.13rem]", className)}
      {...props}
    />
  );
}

export function CardDescription({
  className,
  ...props
}: HTMLAttributes<HTMLParagraphElement>) {
  return (
    <p
      className={cn("text-sm leading-6 text-[var(--ink-soft)]/95", className)}
      {...props}
    />
  );
}
