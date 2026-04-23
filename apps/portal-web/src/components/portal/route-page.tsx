"use client";

import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { portalRouteMap, type PortalRouteKey } from "@/lib/portal-routes";

import { PortalRouteScaffold } from "./route-scaffold";

export function PortalRoutePage({
  routeKey,
  params,
}: {
  routeKey: PortalRouteKey;
  params?: Record<string, string>;
}) {
  const meta = portalRouteMap[routeKey];

  return (
    <PortalRouteScaffold routeKey={routeKey} params={params}>
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardTitle>当前阶段交付</CardTitle>
          <CardDescription>
            本路由已经挂入官方菜单路径、权限元数据和状态预演体系；后续任务会逐步接入真实表单、列表、虚拟滚动和业务操作。
          </CardDescription>
        </Card>
        <Card>
          <CardTitle>关键审计提醒</CardTitle>
          <CardDescription>
            {meta.primaryPermissions.length
              ? `该页面后续关键动作会触发：${meta.primaryPermissions.join(" / ")}，并要求显式审计提示。`
              : "该页面当前以查看链路为主，重点保证主体、角色、租户与作用域可见。"}
          </CardDescription>
        </Card>
      </div>
    </PortalRouteScaffold>
  );
}
