<script lang="ts">
  import type { Prefs, OmarchyTheme } from "../lib/tauri";

  export let open = false;
  export let prefs: Prefs = {
    theme: "omarchy",
    font_size: 15,
    tab_size: 2,
    typewriter: true,
  };
  export let omarchyTheme: OmarchyTheme | null = null;

  $: themes = [
    {
      id: "omarchy",
      label: omarchyTheme
        ? `Omarchy (System: ${omarchyTheme.display_name})`
        : "Omarchy (System)",
    },
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

      <div class="shortcuts-section">
        <div class="shortcuts-header">
          <h3>Keybindings</h3>
          <span class="shortcuts-subtitle">Quick reference</span>
        </div>

        <div class="shortcuts-grid">
          <div class="shortcut-item">
            <span class="desc">Cycle view mode</span>
            <span class="keys"><kbd>Ctrl</kbd><kbd>P</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="desc">Edit / Split / Preview</span>
            <span class="keys"><kbd>Ctrl</kbd><kbd>1 / 2 / 3</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="desc">New document</span>
            <span class="keys"><kbd>Ctrl</kbd><kbd>N</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="desc">Open document</span>
            <span class="keys"><kbd>Ctrl</kbd><kbd>O</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="desc">Save document</span>
            <span class="keys"><kbd>Ctrl</kbd><kbd>S</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="desc">Save As…</span>
            <span class="keys"><kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>S</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="desc">Settings</span>
            <span class="keys"><kbd>Ctrl</kbd><kbd>,</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="desc">Close dialog</span>
            <span class="keys"><kbd>Esc</kbd></span>
          </div>
        </div>

        <div class="math-hint">
          <span class="math-badge">LaTeX</span>
          <span><code>$E=mc^2$</code> inline · <code>$$\int...$$</code> block rendered in Split and Preview</span>
        </div>

        <div class="config-hint">
          <span class="config-label">Config</span>
          <span><code>~/.config/cedilha/config.toml</code> (auto-reloads on save)</span>
        </div>
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
    width: min(520px, 92vw);
    max-height: 88vh;
    overflow-y: auto;
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

  .shortcuts-section {
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  .shortcuts-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .shortcuts-header h3 {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--accent);
  }

  .shortcuts-subtitle {
    font-size: 11px;
    color: var(--fg-dim);
  }

  .shortcuts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px 12px;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    gap: 8px;
  }

  .shortcut-item .desc {
    color: var(--fg);
    font-size: 12px;
  }

  .keys {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  kbd {
    display: inline-block;
    padding: 2px 5px;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    line-height: 1.2;
    color: var(--fg);
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: 3px;
    box-shadow: 0 1px 0 var(--border);
  }

  .math-hint {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 14px;
    padding: 6px 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 11px;
    color: var(--fg-dim);
    line-height: 1.4;
  }

  .math-badge {
    background: var(--accent);
    color: var(--bg);
    font-size: 9px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 3px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .math-hint code {
    font-family: var(--font-mono);
    color: var(--fg);
    background: var(--bg-elev);
    padding: 1px 4px;
    border-radius: 3px;
  }

  .config-hint {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    padding: 6px 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 11px;
    color: var(--fg-dim);
    line-height: 1.4;
  }

  .config-label {
    background: var(--bg-elev);
    color: var(--accent);
    border: 1px solid var(--border);
    font-size: 9px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 3px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .config-hint code {
    font-family: var(--font-mono);
    color: var(--fg);
    background: var(--bg-elev);
    padding: 1px 4px;
    border-radius: 3px;
  }

  @media (max-width: 480px) {
    .shortcuts-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
