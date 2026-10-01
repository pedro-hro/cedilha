<script lang="ts">
  import type { Prefs } from "../lib/tauri";

  export let open = false;
  export let prefs: Prefs = {
    theme: "catppuccin-mocha",
    font_size: 15,
    tab_size: 2,
    wrap: true,
    typewriter: true,
  };

  const themes = [
    { id: "catppuccin-mocha", label: "Catppuccin Mocha" },
    { id: "catppuccin-latte", label: "Catppuccin Latte" },
    { id: "tokyo-night", label: "Tokyo Night" },
    { id: "gruvbox-dark", label: "Gruvbox Dark" },
  ];

  const fontSizes = [12, 13, 14, 15, 16, 17, 18, 20, 22, 24];
  const tabSizes = [1, 2, 3, 4];

  function handleScrimClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      open = false;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="scrim" onclick={handleScrimClick}>
    <div class="panel" role="dialog" aria-label="Settings">
      <header>
        <h2>Settings</h2>
        <button class="icon" aria-label="Close settings" onclick={() => (open = false)}>✕</button>
      </header>

      <div class="row">
        <label for="theme">Theme</label>
        <select id="theme" bind:value={prefs.theme}>
          {#each themes as t}
            <option value={t.id}>{t.label}</option>
          {/each}
        </select>
      </div>

      <div class="row">
        <label for="font">Font size ({prefs.font_size}px)</label>
        <select id="font" bind:value={prefs.font_size}>
          {#each fontSizes as size}
            <option value={size}>{size}px</option>
          {/each}
        </select>
      </div>

      <div class="row">
        <label for="tab">Tab width ({prefs.tab_size})</label>
        <select id="tab" bind:value={prefs.tab_size}>
          {#each tabSizes as size}
            <option value={size}>{size}</option>
          {/each}
        </select>
      </div>

      <div class="row toggle">
        <span>
          <strong>Typewriter scrolling</strong>
          <small>Keep the active line vertically centered.</small>
        </span>
        <input type="checkbox" bind:checked={prefs.typewriter} />
      </div>

      <div class="row toggle">
        <span>
          <strong>Line numbers</strong>
          <small>Display line numbers in the gutter.</small>
        </span>
        <input type="checkbox" bind:checked={prefs.line_numbers} />
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(4px);
  }

  .panel {
    width: min(460px, 90vw);
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px 24px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 700;
    color: var(--accent);
  }

  .icon {
    background: transparent;
    border: none;
    font-size: 16px;
    padding: 4px 8px;
    color: var(--fg-dim);
  }

  .icon:hover {
    color: var(--fg);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid var(--border);
  }

  .row label {
    font-size: 14px;
  }

  .row small {
    color: var(--fg-dim);
    display: block;
    font-size: 12px;
    margin-top: 2px;
  }

  .row select {
    min-width: 150px;
  }

  .row.toggle {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
  }

  .row input[type="checkbox"] {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
    cursor: pointer;
  }
</style>
