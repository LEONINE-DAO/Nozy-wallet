import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  // Required for Tauri bundles: serve assets as relative paths.
  base: "./",
  plugins: [
    react({
      // Disable React Refresh to fix Tauri compatibility issues
      fastRefresh: false,
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Tauri WebView2 is modern Chromium — avoid esbuild downlevel transforms on deps
  // (esbuild 0.28+ no longer lowers destructuring for legacy browser targets).
  esbuild: {
    target: "esnext",
  },
  server: {
    port: 5173,
    strictPort: true,
    host: "localhost",
  },
  build: {
    target: "esnext",
    minify: !process.env.TAURI_DEBUG,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  optimizeDeps: {
    include: ["react", "react-dom"],
    exclude: ["@tauri-apps/api"],
    esbuildOptions: {
      target: "esnext",
    },
  },
});
