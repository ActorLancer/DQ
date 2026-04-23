"use client";

import type { ReactNode } from "react";

import type { PortalRouteKey } from "@/lib/portal-routes";

import { PortalRouteScaffold } from "./route-scaffold";

export function PortalRoutePage({
  routeKey,
  params,
  children,
}: {
  routeKey: PortalRouteKey;
  params?: Record<string, string>;
  children: ReactNode;
}) {
  return (
    <PortalRouteScaffold routeKey={routeKey} params={params}>
      {children}
    </PortalRouteScaffold>
  );
}
