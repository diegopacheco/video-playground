import { defineConfig } from "vite";

const backend = process.env.BACKEND_URL ?? "http://localhost:8104";

export default defineConfig({
  build: { chunkSizeWarningLimit: 1000 },
  server: {
    host: "127.0.0.1",
    port: Number(process.env.FRONTEND_PORT ?? 5104),
    strictPort: true,
    proxy: { "/live": backend },
  },
});
