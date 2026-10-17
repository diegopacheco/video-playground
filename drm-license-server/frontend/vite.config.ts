import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const backend = `http://localhost:${process.env.BACKEND_PORT ?? "8112"}`;

export default defineConfig({
  plugins: [react()],
  build: { chunkSizeWarningLimit: 1200 },
  server: {
    port: Number(process.env.FRONTEND_PORT ?? "5112"),
    strictPort: true,
    proxy: {
      "/api": backend,
      "/media": backend,
    },
  },
});
