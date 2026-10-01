import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

// @ts-check
export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "es2021",
    minify: false,
  },
});
