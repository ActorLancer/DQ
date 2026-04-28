import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatPrice(cents: number, model: string) {
  return `${(cents / 100).toLocaleString("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 0,
  })} / ${model}`;
}

export function labelFromKey(key: string) {
  return key
    .split("_")
    .map((item) => item[0].toUpperCase() + item.slice(1))
    .join(" ");
}
