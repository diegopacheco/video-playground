import { defineConfig } from "vite";

const port = Number(process.env.FRONTEND_PORT ?? 5109);
const backend = process.env.BACKEND_URL ?? "http://127.0.0.1:8109";

export default defineConfig({
  server: {
    port,
    strictPort: true,
    host: "127.0.0.1",
    proxy: {
      "/api": { target: backend },
    },
  },
});
