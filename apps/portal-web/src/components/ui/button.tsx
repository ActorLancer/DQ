"use client";

import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-full px-4 py-2 text-sm font-medium transition-colors disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default:
          "bg-[var(--accent-strong)] text-white shadow-lg shadow-[color:var(--accent-shadow)] hover:bg-[var(--accent-stronger)]",
        secondary:
          "bg-white/80 text-[var(--ink-strong)] ring-1 ring-black/10 hover:bg-white",
        ghost:
          "bg-transparent text-[var(--ink-soft)] hover:bg-black/5 hover:text-[var(--ink-strong)]",
        warning:
          "bg-[var(--warning-soft)] text-[var(--warning-ink)] ring-1 ring-[var(--warning-ring)] hover:bg-[var(--warning-strong)]/25",
      },
      size: {
        default: "h-10",
        sm: "h-8 px-3 text-xs",
        lg: "h-12 px-5 text-base",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  },
);
Button.displayName = "Button";

export { Button, buttonVariants };
