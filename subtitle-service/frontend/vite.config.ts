import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const backend = process.env.BACKEND_URL ?? "http://localhost:8106";
const isolation = {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp",
};

export default defineConfig({
  plugins: [react()],
  optimizeDeps: {
    exclude: ["jassub"],
    include: ["jassub > throughput", "jassub > abslink", "jassub > abslink/w3c", "jassub > rvfc-polyfill", "jassub > lfa-ponyfill"],
  },
  server: {
    host: "127.0.0.1",
    port: Number(process.env.FRONTEND_PORT ?? 5106),
    strictPort: true,
    headers: isolation,
    proxy: {
      "/api": backend,
      "/hls": backend,
      "/media": backend,
    },
  },
  preview: { headers: isolation },
});
