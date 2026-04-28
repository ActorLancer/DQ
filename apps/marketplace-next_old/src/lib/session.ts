import { cookies } from "next/headers";

const SESSION_COOKIE_NAME = "datab_marketplace_session";

type MarketplaceSession =
  | { mode: "guest" }
  | { mode: "bearer"; accessToken: string }
  | { mode: "local"; loginId: string; role: string; tenantId?: string };

export async function readMarketplaceSession(): Promise<MarketplaceSession> {
  const cookieStore = await cookies();
  const raw = cookieStore.get(SESSION_COOKIE_NAME)?.value;
  if (!raw) {
    return { mode: "guest" };
  }

  try {
    const decoded = JSON.parse(Buffer.from(raw, "base64url").toString("utf8"));
    if (decoded.mode === "bearer" && typeof decoded.accessToken === "string") {
      return { mode: "bearer", accessToken: decoded.accessToken };
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

export function buildSessionHeaders(session: MarketplaceSession): HeadersInit {
  if (session.mode === "bearer") {
    return { authorization: `Bearer ${session.accessToken}` };
  }
  if (session.mode === "local") {
    return {
      "x-login-id": session.loginId,
      "x-role": session.role,
      ...(session.tenantId ? { "x-tenant-id": session.tenantId } : {}),
    };
  }
  return {};
}
