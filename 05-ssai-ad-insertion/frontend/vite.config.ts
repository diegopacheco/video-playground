import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const backend = `http://localhost:${process.env.BACKEND_PORT ?? "8105"}`;

export default defineConfig({
  plugins: [react()],
  build: { chunkSizeWarningLimit: 1000 },
  server: {
    port: Number(process.env.FRONTEND_PORT ?? "5105"),
    strictPort: true,
    proxy: {
      "/api": backend,
      "/media": backend,
    },
  },
});
