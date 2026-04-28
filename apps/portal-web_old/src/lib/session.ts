import { cookies } from "next/headers";

const SESSION_COOKIE_NAME = "datab_portal_session";

export interface PortalSessionPreview {
  source: "claims" | "local_cookie";
  mode: "jwt_mirror" | "local_test_user";
  user_id?: string;
  org_id?: string;
  login_id?: string;
  display_name?: string;
  tenant_id?: string;
  roles: string[];
  auth_context_level: string;
  exp?: number;
}

export type PortalSession =
  | {
      mode: "guest";
    }
  | {
      mode: "bearer";
      accessToken: string;
      label?: string;
    }
  | {
      mode: "local";
      loginId: string;
      role: string;
      tenantId?: string;
    };

export async function readPortalSession(): Promise<PortalSession> {
  const cookieStore = await cookies();
  const raw = cookieStore.get(SESSION_COOKIE_NAME)?.value;
  if (!raw) {
    return { mode: "guest" };
  }

  try {
    const decoded = JSON.parse(Buffer.from(raw, "base64url").toString("utf8"));
    if (decoded.mode === "bearer" && typeof decoded.accessToken === "string") {
      return {
        mode: "bearer",
        accessToken: decoded.accessToken,
        label: typeof decoded.label === "string" ? decoded.label : undefined,
      };
    }
    if (
      decoded.mode === "local" &&
      typeof decoded.loginId === "string" &&
      typeof decoded.role === "string"
    ) {
      return {
        mode: "local",
        loginId: decoded.loginId,
        role: decoded.role,
        tenantId: typeof decoded.tenantId === "string" ? decoded.tenantId : undefined,
      };
    }
  } catch {
    return { mode: "guest" };
  }

  return { mode: "guest" };
}

export async function writePortalSession(session: Exclude<PortalSession, { mode: "guest" }>) {
  const cookieStore = await cookies();
  const payload = Buffer.from(JSON.stringify(session), "utf8").toString(
    "base64url",
  );
  cookieStore.set(SESSION_COOKIE_NAME, payload, {
    httpOnly: true,
    sameSite: "lax",
    secure: false,
    path: "/",
    maxAge: 60 * 60 * 12,
  });
}

export async function clearPortalSession() {
  const cookieStore = await cookies();
  cookieStore.delete(SESSION_COOKIE_NAME);
}

export function readPortalSessionPreview(
  session: PortalSession,
): PortalSessionPreview | null {
  if (session.mode === "local") {
    const tenantId = session.tenantId ?? inferLocalSessionTenantId(session.loginId, session.role);
    return {
      source: "local_cookie",
      mode: "local_test_user",
      org_id: tenantId,
      login_id: session.loginId,
      display_name: session.loginId,
      tenant_id: tenantId,
      roles: [session.role],
      auth_context_level: "aal1",
    };
  }

  if (session.mode !== "bearer") {
    return null;
  }

  const payload = decodeBearerPayload(session.accessToken);
  if (!payload) {
    return null;
  }

  const roles = readStringArray(
    readRecord(payload.realm_access)?.roles,
  );
  if (!roles.length) {
    return null;
  }

  const userId = readString(payload.user_id);
  const orgId = readString(payload.org_id);
  if (!userId || !orgId) {
    return null;
  }

  return {
    source: "claims",
    mode: "jwt_mirror",
    user_id: userId,
    org_id: orgId,
    login_id: readString(payload.preferred_username),
    display_name: readString(payload.name) ?? readString(payload.preferred_username),
    tenant_id: orgId,
    roles,
    auth_context_level: "aal1",
    exp: typeof payload.exp === "number" ? payload.exp : undefined,
  };
}

export function buildSessionHeaders(session: PortalSession): HeadersInit {
  if (session.mode === "bearer") {
    const preview = readPortalSessionPreview(session);
    return {
      authorization: `Bearer ${session.accessToken}`,
      ...(preview
        ? {
            "x-user-id": preview.user_id ?? "",
            "x-tenant-id": preview.tenant_id ?? preview.org_id ?? "",
            "x-role": preview.roles[0] ?? "",
          }
        : {}),
    };
  }

  if (session.mode === "local") {
    const tenantId = session.tenantId ?? inferLocalSessionTenantId(session.loginId, session.role);
    return {
      "x-login-id": session.loginId,
      "x-role": session.role,
      ...(tenantId ? { "x-tenant-id": tenantId } : {}),
    };
  }

  return {};
}

export function inferLocalSessionTenantId(loginId: string, role: string): string | undefined {
  if (
    loginId.includes("seller") ||
    role === "seller_operator" ||
    role === "seller_storage_operator"
  ) {
    return "10000000-0000-0000-0000-000000000101";
  }
  if (role.startsWith("platform_") || loginId.includes("ops") || loginId.includes("auditor")) {
    return "10000000-0000-0000-0000-000000000103";
  }
  if (
    loginId.includes("buyer") ||
    loginId.includes("developer") ||
    role === "buyer_operator" ||
    role === "procurement_manager" ||
    role === "tenant_admin" ||
    role === "tenant_developer"
  ) {
    return "10000000-0000-0000-0000-000000000102";
  }
  return undefined;
}

function decodeBearerPayload(accessToken: string): Record<string, unknown> | null {
  const [, payload] = accessToken.split(".");
  if (!payload) {
    return null;
  }

  try {
    const decoded = JSON.parse(
      Buffer.from(payload, "base64url").toString("utf8"),
    );
    return readRecord(decoded) ?? null;
  } catch {
    return null;
  }
}

function readRecord(value: unknown): Record<string, unknown> | undefined {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return undefined;
}

function readString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function readStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.filter((item): item is string => typeof item === "string");
}
