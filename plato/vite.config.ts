import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

/**
 * Vite Configuration for Plato
 * Optimized for React 19 and Tailwind v4. 
 * Establishes a high-performance HMR bridge for Tauri v2.
 */
export default defineConfig(async () => ({
  plugins: [
    react(),
    tailwindcss(),
  ],

  // Vite options tailored for Tauri development and navigation.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "0.0.0.0",
    hmr: {
      protocol: "ws",
      host: "0.0.0.0",
      port: 1421,
    },
  },
}));