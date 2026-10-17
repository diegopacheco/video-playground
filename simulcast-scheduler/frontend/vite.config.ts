import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const backend = process.env.BACKEND_URL ?? "http://localhost:8107";
const port = Number(process.env.FRONTEND_PORT ?? 5107);

export default defineConfig({
  plugins: [react()],
  server: {
    port,
    strictPort: true,
    proxy: { "/api": backend },
  },
});
