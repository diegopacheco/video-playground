import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const backend = process.env.BACKEND_PORT ?? "8115";
const port = Number(process.env.FRONTEND_PORT ?? "5115");

export default defineConfig({
  plugins: [react()],
  server: {
    port,
    strictPort: true,
    proxy: { "/api": `http://localhost:${backend}` },
  },
});
