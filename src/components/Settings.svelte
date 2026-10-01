<script lang="ts">
  import type { Prefs, OmarchyTheme } from "../lib/tauri";
  import {
    ACTIONS,
    resolveBindings,
    comboFromEvent,
    comboId,
    comboParts,
    type ActionId,
  } from "../lib/keybindings";

  export let open = false;
  export let prefs: Prefs = {
    theme: "omarchy",
    font_size: 15,
    tab_size: 2,
    typewriter: true,
    line_numbers: true,
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

  const widths = [680, 800, 900, 1040, 1200, 1400];

  // ----- keybindings -----
  $: bindings = resolveBindings(prefs.keybindings);
  let recording: ActionId | null = null;
  let recordError = "";

  const sameCombos = (a: string[], b: string[]) =>
    a.length === b.length && a.every((c, i) => comboId(c) === comboId(b[i]));

  function setBinding(id: ActionId, combos: string[] | null) {
    const next = { ...(prefs.keybindings ?? {}) };
    const def = ACTIONS.find((a) => a.id === id)!.defaults;
    if (combos === null || sameCombos(combos, def)) delete next[id];
    else next[id] = combos;
    prefs.keybindings = next;
  }

  function recordKey(e: KeyboardEvent) {
    // Capture everything while recording so the press doesn't also trigger the shortcut.
    e.preventDefault();
    e.stopImmediatePropagation();
    if (e.key === "Escape") {
      stopRecording();
      return;
    }
    const combo = comboFromEvent(e);
    if (!combo) {
      if (!["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
        recordError = "Use Ctrl or Alt with a key (function keys work alone).";
      }
      return;
    }
    const id = recording!;
    const wanted = comboId(combo);
    const clash = ACTIONS.find(
      (a) => a.id !== id && bindings[a.id].some((c) => comboId(c) === wanted),
    );
    if (clash) {
      recordError = `${combo} is already used by "${clash.label}".`;
      return;
    }
    setBinding(id, [combo]);
    stopRecording();
  }

  function startRecording(id: ActionId) {
    stopRecording();
    recording = id;
    recordError = "";
    window.addEventListener("keydown", recordKey, true);
  }

  function stopRecording() {
    window.removeEventListener("keydown", recordKey, true);
    recording = null;
  }

  $: if (!open && recording) stopRecording();

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

      <div class="row">
        <label for="fontfamily">Font family</label>
        <input
          id="fontfamily"
          type="text"
          placeholder="JetBrains Mono"
          value={prefs.font_family ?? ""}
          onchange={(e) => (prefs.font_family = e.currentTarget.value.trim() || "JetBrains Mono")}
        />
      </div>

      <div class="row">
        <label for="width">Text column width ({prefs.editor_max_width ?? 1040}px)</label>
        <select id="width" bind:value={prefs.editor_max_width}>
          {#each widths as w}
            <option value={w}>{w}px</option>
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
          <span class="shortcuts-subtitle">Click a shortcut, then press the new keys</span>
        </div>

        <div class="bind-list">
          {#each ACTIONS as action (action.id)}
            {@const custom = !!prefs.keybindings?.[action.id]}
            <div class="bind-row">
              <span class="desc">{action.label}</span>
              <span class="bind-controls">
                <button
                  class="keys-btn"
                  class:recording={recording === action.id}
                  class:custom
                  onclick={() => (recording === action.id ? stopRecording() : startRecording(action.id))}
                  title="Change shortcut"
                >
                  {#if recording === action.id}
                    <span class="press">Press keys… (Esc cancels)</span>
                  {:else}
                    {#each bindings[action.id] as combo, i}
                      {#if i > 0}<span class="or">or</span>{/if}
                      <span class="keys">
                        {#each comboParts(combo) as part}<kbd>{part}</kbd>{/each}
                      </span>
                    {/each}
                  {/if}
                </button>
                <button
                  class="reset"
                  aria-label="Reset to default"
                  title="Reset to default"
                  disabled={!custom}
                  onclick={() => setBinding(action.id, null)}
                >
                  ↺
                </button>
              </span>
            </div>
          {/each}
        </div>

        {#if recordError}
          <div class="bind-error" role="alert">{recordError}</div>
        {/if}

        <div class="fixed-keys">
          <span class="fixed-title">Fixed</span>
          <span><kbd>Enter</kbd> / <kbd>Shift</kbd><kbd>Enter</kbd> next / previous match in the search box ·
            <kbd>Esc</kbd> close search or this dialog ·
            <kbd>Ctrl</kbd><kbd>D</kbd> select next occurrence ·
            <kbd>Ctrl</kbd><kbd>Alt</kbd><kbd>G</kbd> go to line</span>
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

  .bind-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .bind-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 4px 8px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .bind-row .desc {
    color: var(--fg);
    font-size: 12px;
  }

  .bind-controls {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .keys-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-width: 90px;
    justify-content: center;
    padding: 3px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius);
    cursor: pointer;
  }

  .keys-btn:hover {
    border-color: var(--border);
  }

  .keys-btn.custom kbd {
    border-color: var(--accent);
  }

  .keys-btn.recording {
    border-color: var(--accent);
    background: var(--bg-elev);
  }

  .press {
    font-size: 11px;
    color: var(--accent);
  }

  .or {
    font-size: 10px;
    color: var(--fg-dim);
  }

  .reset {
    background: transparent;
    border: none;
    color: var(--fg-dim);
    cursor: pointer;
    padding: 2px 6px;
    font-size: 14px;
  }

  .reset:hover:not(:disabled) {
    color: var(--accent);
  }

  .reset:disabled {
    opacity: 0.25;
    cursor: default;
  }

  .bind-error {
    margin-top: 8px;
    padding: 6px 10px;
    font-size: 11px;
    color: var(--danger);
    border: 1px solid var(--danger);
    border-radius: var(--radius);
  }

  .fixed-keys {
    margin-top: 10px;
    font-size: 11px;
    color: var(--fg-dim);
    line-height: 1.8;
  }

  .fixed-title {
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 9px;
    margin-right: 6px;
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

  .row input[type="text"] {
    min-width: 150px;
    background: var(--bg-surface);
    color: var(--fg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 5px 8px;
  }
</style>
