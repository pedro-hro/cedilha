import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

// Minimal static adapter: Tauri serves the built SPA straight from `build/`.
// adapter-static isn't available offline, so replicate its core behavior with
// the builder helpers SvelteKit already exposes (writeClient + writePrerendered).
const adapter = {
  name: "cedilha-static",
  adapt(builder) {
    builder.rimraf("build");
    builder.writeClient("build");
    builder.writePrerendered("build");
  },
};

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter,
    alias: {
      $components: "src/components",
      $lib: "src/lib",
    },
  },
};

export default config;
