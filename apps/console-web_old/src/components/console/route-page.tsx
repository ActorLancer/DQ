"use client";

import type { ReactNode } from "react";

import type { ConsoleRouteKey } from "@/lib/console-routes";

import { ConsoleRouteScaffold } from "./route-scaffold";

export function ConsoleRoutePage({
  routeKey,
  params,
  children,
}: {
  routeKey: ConsoleRouteKey;
  params?: Record<string, string>;
  children: ReactNode;
}) {
  return (
    <ConsoleRouteScaffold routeKey={routeKey} params={params}>
      {children}
    </ConsoleRouteScaffold>
  );
}
