import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const port = Number(process.env.FRONTEND_PORT ?? "5111");
const backend = process.env.BACKEND_URL ?? "http://127.0.0.1:8111";

export default defineConfig({
  plugins: [react()],
  server: {
    port,
    strictPort: true,
    host: "127.0.0.1",
    proxy: { "/api": backend },
  },
});
