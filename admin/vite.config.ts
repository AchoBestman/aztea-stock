import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { r2UploadPlugin } from "./vite-r2-plugin";
import path from "path";

/** Web dev server; same src can be wrapped in Tauri later (HashRouter, no SSR). */
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  return {
    plugins: [react(), tailwindcss(), r2UploadPlugin()],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
    clearScreen: false,
    server: {
      port: Number(env.VITE_DEV_PORT || 5173),
      strictPort: true,
    },
    preview: {
      port: Number(env.VITE_PREVIEW_PORT || 4173),
    },
    build: {
      outDir: "dist",
      sourcemap: true,
    },
  };
});
