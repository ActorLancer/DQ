import { cookies } from "next/headers";

const SESSION_COOKIE_NAME = "datab_console_session";

export type ConsoleSession =
  | {
      mode: "guest";
    }
  | {
      mode: "bearer";
      accessToken: string;
      label?: string;
      userId?: string;
      tenantId?: string;
      role?: string;
    }
  | {
      mode: "local";
      loginId: string;
      role: string;
      userId?: string;
      tenantId?: string;
    };

export async function readConsoleSession(): Promise<ConsoleSession> {
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
        userId: typeof decoded.userId === "string" ? decoded.userId : undefined,
        tenantId: typeof decoded.tenantId === "string" ? decoded.tenantId : undefined,
        role: typeof decoded.role === "string" ? decoded.role : undefined,
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
        userId: typeof decoded.userId === "string" ? decoded.userId : undefined,
        tenantId: typeof decoded.tenantId === "string" ? decoded.tenantId : undefined,
      };
    }
  } catch {
    return { mode: "guest" };
  }

  return { mode: "guest" };
}

export async function writeConsoleSession(
  session: Exclude<ConsoleSession, { mode: "guest" }>,
) {
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

export async function clearConsoleSession() {
  const cookieStore = await cookies();
  cookieStore.delete(SESSION_COOKIE_NAME);
}

export function buildSessionHeaders(session: ConsoleSession): HeadersInit {
  if (session.mode === "bearer") {
    return {
      authorization: `Bearer ${session.accessToken}`,
      ...(session.userId ? { "x-user-id": session.userId } : {}),
      ...(session.tenantId ? { "x-tenant-id": session.tenantId } : {}),
      ...(session.role ? { "x-role": session.role } : {}),
    };
  }

  if (session.mode === "local") {
    return {
      "x-login-id": session.loginId,
      "x-role": session.role,
      ...(session.userId ? { "x-user-id": session.userId } : {}),
      ...(session.tenantId ? { "x-tenant-id": session.tenantId } : {}),
    };
  }

  return {};
}
