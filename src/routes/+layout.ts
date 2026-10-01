// The layout is prerendered to a static index.html that Tauri serves. The
// application shell then hydrates on the client from +page.svelte.
export const prerender = true;
export const ssr = false;

// No data loading is required for the layout; the application shell applies
// its own theme from app.css.
