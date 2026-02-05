import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
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
  server: {
    port: 5173,
    strictPort: true,
    host: "localhost",
  },
  build: {
    target: "es2020",
    minify: !process.env.TAURI_DEBUG,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  // Fix React Refresh in Tauri
  optimizeDeps: {
    include: ["react", "react-dom"],
    exclude: ["@tauri-apps/api"],
  },
});
