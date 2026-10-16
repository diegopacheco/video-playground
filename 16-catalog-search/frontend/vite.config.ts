import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const port = Number(process.env.FRONTEND_PORT ?? 5116);
const backend = `http://127.0.0.1:${process.env.BACKEND_PORT ?? 8116}`;

export default defineConfig({
  plugins: [react()],
  server: { port, strictPort: true, proxy: { "/api": backend } },
  preview: { port, strictPort: true, proxy: { "/api": backend } },
});
