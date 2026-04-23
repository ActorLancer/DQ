"use client";

import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { consoleRouteMap, type ConsoleRouteKey } from "@/lib/console-routes";

import { ConsoleRouteScaffold } from "./route-scaffold";

export function ConsoleRoutePage({
  routeKey,
  params,
}: {
  routeKey: ConsoleRouteKey;
  params?: Record<string, string>;
}) {
  const meta = consoleRouteMap[routeKey];

  return (
    <ConsoleRouteScaffold routeKey={routeKey} params={params}>
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardTitle>当前阶段交付</CardTitle>
          <CardDescription>
            本路由已经挂入官方控制台菜单路径、权限元数据和状态预演体系；后续任务会逐步接入真实联查、列表、表单和高风险动作。
          </CardDescription>
        </Card>
        <Card>
          <CardTitle>关键审计提醒</CardTitle>
          <CardDescription>
            {meta.primaryPermissions.length
              ? `该页面后续关键动作会触发：${meta.primaryPermissions.join(" / ")}，并要求显式审计提示。`
              : "该页面当前以查看链路为主，重点保证主体、角色、租户、作用域与联查键可见。"}
          </CardDescription>
        </Card>
      </div>
    </ConsoleRouteScaffold>
  );
}
