import { cookies } from "next/headers";

const SESSION_COOKIE_NAME = "datab_portal_session";

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

export function buildSessionHeaders(session: PortalSession): HeadersInit {
  if (session.mode === "bearer") {
    return {
      authorization: `Bearer ${session.accessToken}`,
    };
  }

  if (session.mode === "local") {
    return {
      "x-login-id": session.loginId,
      "x-role": session.role,
    };
  }

  return {};
}
