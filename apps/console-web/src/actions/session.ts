"use server";

import { revalidatePath } from "next/cache";
import { z } from "zod";

import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";

import { createServerSdk } from "@/lib/platform-sdk";
import {
  clearConsoleSession,
  type ConsoleSession,
  writeConsoleSession,
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
  }),
]);

export interface SessionActionState {
  ok: boolean;
  message: string;
}

export async function connectConsoleSession(
  payload: unknown,
): Promise<SessionActionState> {
  const parsed = connectSessionSchema.safeParse(payload);
  if (!parsed.success) {
    return {
      ok: false,
      message: parsed.error.issues[0]?.message ?? "登录态占位校验失败",
    };
  }

  const session: Exclude<ConsoleSession, { mode: "guest" }> =
    parsed.data.mode === "bearer"
      ? {
          mode: "bearer",
          accessToken: parsed.data.accessToken,
          label: parsed.data.label || "Control Plane Token",
        }
      : {
          mode: "local",
          loginId: parsed.data.loginId,
          role: parsed.data.role,
        };

  const sdk = createServerSdk(
    session.mode === "bearer"
      ? {
          authorization: `Bearer ${session.accessToken}`,
        }
      : {
          "x-login-id": session.loginId,
          "x-role": session.role,
        },
  );

  try {
    const auth = await sdk.iam.getAuthMe();
    const verifiedSession: Exclude<ConsoleSession, { mode: "guest" }> =
      session.mode === "bearer"
        ? {
            ...session,
            userId: auth.data.user_id ?? undefined,
            tenantId: auth.data.tenant_id ?? auth.data.org_id ?? undefined,
            role: auth.data.roles[0],
          }
        : {
            ...session,
            userId: auth.data.user_id ?? undefined,
            tenantId: auth.data.tenant_id ?? auth.data.org_id ?? undefined,
          };
    await writeConsoleSession(verifiedSession);
    revalidatePath("/", "layout");

    return {
      ok: true,
      message:
        session.mode === "bearer"
          ? "控制台 Bearer 会话已验证并写入 HttpOnly Cookie。"
          : `控制台本地测试身份已切换为 ${session.loginId} / ${session.role}。`,
    };
  } catch (error) {
    return {
      ok: false,
      message: formatPlatformErrorForDisplay(error, {
        fallbackCode: "UNAUTHORIZED",
        fallbackDescription: "无法验证控制台登录态占位，请检查 platform-core、Keycloak 和当前会话配置。",
      }),
    };
  }
}

export async function disconnectConsoleSession(): Promise<SessionActionState> {
  await clearConsoleSession();
  revalidatePath("/", "layout");
  return {
    ok: true,
    message: "控制台会话已清空。",
  };
}
