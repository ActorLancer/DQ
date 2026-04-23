"use server";

import { revalidatePath } from "next/cache";
import { z } from "zod";

import { PlatformApiError } from "@datab/sdk-ts";

import { createServerSdk } from "@/lib/platform-sdk";
import {
  buildSessionHeaders,
  clearPortalSession,
  readPortalSessionPreview,
  type PortalSession,
  writePortalSession,
} from "@/lib/session";

const connectSessionSchema = z.discriminatedUnion("mode", [
  z.object({
    mode: z.literal("bearer"),
    accessToken: z.string().min(20, "请输入有效的 Bearer Token"),
    label: z.string().max(48).optional().default(""),
  }),
  z.object({
    mode: z.literal("local"),
    loginId: z.string().min(1, "请输入本地测试 login_id"),
    role: z.string().min(1, "请选择本地测试角色"),
    tenantId: z
      .string()
      .regex(
        /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
        "请输入本地测试租户 UUID",
      )
      .optional(),
  }),
]);

export interface SessionActionState {
  ok: boolean;
  message: string;
}

export async function connectPortalSession(
  payload: unknown,
): Promise<SessionActionState> {
  const parsed = connectSessionSchema.safeParse(payload);
  if (!parsed.success) {
    return {
      ok: false,
      message: parsed.error.issues[0]?.message ?? "登录态占位校验失败",
    };
  }

  const session: Exclude<PortalSession, { mode: "guest" }> =
    parsed.data.mode === "bearer"
      ? {
          mode: "bearer",
          accessToken: parsed.data.accessToken,
          label: parsed.data.label || "手工接入令牌",
        }
      : {
          mode: "local",
          loginId: parsed.data.loginId,
          role: parsed.data.role,
          tenantId: parsed.data.tenantId,
        };

  const preview = readPortalSessionPreview(session);
  if (session.mode === "bearer") {
    if (!preview) {
      return {
        ok: false,
        message: "Bearer Token 缺少可识别的 user_id / org_id / roles claims。",
      };
    }
    if (preview.exp && preview.exp * 1000 <= Date.now()) {
      return {
        ok: false,
        message: "Bearer Token 已过期，请重新获取 Keycloak / IAM access token。",
      };
    }
  }

  const sdk = createServerSdk(buildSessionHeaders(session));

  try {
    if (session.mode === "bearer") {
      const roles = preview?.roles ?? [];
      if (roles.some((role) => role === "platform_admin" || role === "buyer_operator")) {
        await sdk.search.searchCatalog({
          q: "工业",
          entity_scope: "all",
          page: 1,
          page_size: 1,
        });
      } else if (
        preview?.tenant_id &&
        roles.some((role) => role === "tenant_admin" || role === "seller_operator")
      ) {
        await sdk.recommendation.getRecommendations({
          placement_code: "home_featured",
          subject_scope: "organization",
          subject_org_id: preview.tenant_id,
          limit: 1,
        });
      } else {
        await sdk.iam.getAuthMe();
      }

      if (
        roles.some(
          (role) =>
            role === "platform_admin" ||
            role === "tenant_admin",
        )
      ) {
        await sdk.catalog.getStandardScenarioTemplates();
      }
    } else {
      await sdk.iam.getAuthMe();
    }
    await writePortalSession(session);
    revalidatePath("/", "layout");

    return {
      ok: true,
      message:
        session.mode === "bearer"
          ? "Bearer 会话已验证并写入 HttpOnly Cookie。"
          : `本地测试身份已切换为 ${session.loginId} / ${session.role}。`,
    };
  } catch (error) {
    if (error instanceof PlatformApiError) {
      return {
        ok: false,
        message: `${error.code}: ${error.message}`,
      };
    }

    return {
      ok: false,
      message:
        error instanceof Error
          ? `无法验证当前登录态占位：${error.message}`
          : "无法验证当前登录态占位，请检查 platform-core 或 Keycloak 配置。",
    };
  }
}

export async function disconnectPortalSession(): Promise<SessionActionState> {
  await clearPortalSession();
  revalidatePath("/", "layout");
  return {
    ok: true,
    message: "门户会话已清空。",
  };
}
