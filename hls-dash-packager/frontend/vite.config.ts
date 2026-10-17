import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  build: { chunkSizeWarningLimit: 2000 },
  server: { port: Number(process.env.FRONTEND_PORT ?? 5101), strictPort: true },
  preview: { port: Number(process.env.FRONTEND_PORT ?? 5101), strictPort: true },
});
