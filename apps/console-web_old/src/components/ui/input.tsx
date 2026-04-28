import * as React from "react";

import { cn } from "@/lib/utils";

export type InputProps = React.InputHTMLAttributes<HTMLInputElement>;

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, ...props }, ref) => {
    return (
      <input
        ref={ref}
        className={cn(
          "flex h-11 w-full rounded-2xl border border-[var(--surface-ring)] bg-[var(--surface-strong)] px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)] focus:shadow-[0_0_0_4px_rgba(165,79,30,0.1)]",
          className,
        )}
        {...props}
      />
    );
  },
);
Input.displayName = "Input";
