import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

/**
 * Vite Configuration for Plato
 * Integrated with Tailwind v4 for high-performance styling and 
 * optimized for Tauri's IPC requirements.
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