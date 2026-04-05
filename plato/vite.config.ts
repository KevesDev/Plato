import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

/**
 * Vite Configuration for Plato
 * Optimized for React 19 and Tailwind v4. 
 * Establishes a high-performance HMR bridge for Tauri v2 using local loopback.
 */
export default defineConfig(async () => ({
  plugins: [
    react(),
    tailwindcss(),
  ],

  // Vite options tailored for Tauri development.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1", // Corrected for Windows/Tauri resolution
    hmr: {
      protocol: "ws",
      host: "127.0.0.1", // Corrected for Windows/Tauri resolution
      port: 1421,
    },
  },
}));