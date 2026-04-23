const FALLBACK_PLATFORM_CORE_BASE_URL = "http://127.0.0.1:8094";

export function getPlatformCoreBaseUrl() {
  return (
    process.env.PLATFORM_CORE_BASE_URL ?? FALLBACK_PLATFORM_CORE_BASE_URL
  ).replace(/\/$/, "");
}
