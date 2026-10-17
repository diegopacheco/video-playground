import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const backendPort = process.env.BACKEND_PORT ?? "8114";
const frontendPort = Number(process.env.FRONTEND_PORT ?? "5114");

export default defineConfig({
  plugins: [react()],
  server: {
    port: frontendPort,
    strictPort: true,
    proxy: {
      "/ws": { target: `ws://127.0.0.1:${backendPort}`, ws: true },
    },
  },
});
