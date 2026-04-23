import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: ["@datab/sdk-ts"],
  typedRoutes: true,
};

export default nextConfig;
