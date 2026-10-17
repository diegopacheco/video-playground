import { defineConfig } from "vite";

const port = Number(process.env.FRONTEND_PORT ?? 5103);
const edge = process.env.EDGE_URL ?? "http://127.0.0.1:6103";
const origin = process.env.ORIGIN_URL ?? "http://127.0.0.1:8103";

export default defineConfig({
  server: {
    port,
    strictPort: true,
    host: "127.0.0.1",
    proxy: {
      "/edge": { target: edge, rewrite: (path) => path.replace(/^\/edge/, "") },
      "/origin": { target: origin, rewrite: (path) => path.replace(/^\/origin/, "") },
    },
  },
});
