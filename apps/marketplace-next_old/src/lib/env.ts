const FALLBACK_PLATFORM_CORE_BASE_URL = "http://127.0.0.1:8094";

export function getPlatformCoreBaseUrl() {
  return (
    process.env.PLATFORM_CORE_BASE_URL ?? FALLBACK_PLATFORM_CORE_BASE_URL
  ).replace(/\/$/, "");
}

export function isLiveDataEnabled() {
  return process.env.NEXT_PUBLIC_MARKETPLACE_LIVE_DATA === "1";
}
