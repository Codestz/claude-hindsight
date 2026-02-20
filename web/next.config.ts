import type { NextConfig } from "next";

const isDev = process.env.NODE_ENV !== "production";

const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: { unoptimized: true },
  // Rewrites only apply in `next dev` (dev server mode).
  // During `next build` (static export) these are silently ignored.
  // In dev, this proxies /api/* → Rust server at :7227.
  ...(isDev && {
    async rewrites() {
      return [
        {
          source: "/api/:path*",
          destination: "http://localhost:7227/api/:path*",
        },
      ];
    },
  }),
};

export default nextConfig;
