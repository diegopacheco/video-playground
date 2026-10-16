import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const backend = `http://localhost:${process.env.BACKEND_PORT ?? 8102}`;
const port = Number(process.env.FRONTEND_PORT ?? 5102);

export default defineConfig({
  plugins: [react()],
  server: { port, strictPort: true, proxy: { "/api": backend } },
  preview: { port, strictPort: true, proxy: { "/api": backend } },
});
