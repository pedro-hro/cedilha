<script lang="ts">
  import { tick } from "svelte";
  import type { Prefs, OmarchyTheme } from "../lib/tauri";
  import {
    ACTIONS,
    resolveBindings,
    comboFromEvent,
    comboId,
    comboParts,
    type ActionDef,
    type ActionId,
  } from "../lib/keybindings";

  export let open = false;
  export let prefs: Prefs = {
    theme: "gruvbox-dark",
    font_size: 15,
    tab_size: 2,
    typewriter: true,
    line_numbers: true,
  };
  export let omarchyTheme: OmarchyTheme | null = null;

  type Choice = { value: string | number; label: string };
  type PrefKey =
    | "theme"
    | "font_size"
    | "tab_size"
    | "editor_max_width"
    | "default_view"
    | "window_controls"
    | "rounded_corners";
  type BoolKey = "typewriter" | "line_numbers";
  type Row =
    | { kind: "choice"; section: string; label: string; key: PrefKey; options: Choice[] }
    | { kind: "toggle"; section: string; label: string; hint: string; key: BoolKey }
    | { kind: "text"; section: string; label: string }
    | { kind: "key"; section: string; action: ActionDef };

  $: themes = [
    {
      value: "omarchy",
      label: omarchyTheme ? `omarchy (${omarchyTheme.display_name})` : "omarchy (system)",
    },
    { value: "catppuccin-mocha", label: "catppuccin mocha" },
    { value: "catppuccin-latte", label: "catppuccin latte" },
    { value: "tokyo-night", label: "tokyo night" },
    { value: "gruvbox-dark", label: "gruvbox dark" },
  ];

  const sizeChoices = (xs: number[], unit = "px"): Choice[] =>
    xs.map((v) => ({ value: v, label: `${v}${unit}` }));

  $: rows = [
    { kind: "choice", section: "appearance", label: "Theme", key: "theme", options: themes },
    { kind: "text", section: "appearance", label: "Font family" },
    {
      kind: "choice",
      section: "appearance",
      label: "Font size",
      key: "font_size",
      options: sizeChoices([12, 13, 14, 15, 16, 17, 18, 20, 22, 24]),
    },
    {
      kind: "choice",
      section: "appearance",
      label: "Text column width",
      key: "editor_max_width",
      options: sizeChoices([680, 800, 900, 1040, 1200, 1400]),
    },
    {
      kind: "choice",
      section: "editor",
      label: "Tab width",
      key: "tab_size",
      options: sizeChoices([1, 2, 3, 4], ""),
    },
    {
      kind: "toggle",
      section: "editor",
      label: "Typewriter scrolling",
      hint: "keep the active line centered",
      key: "typewriter",
    },
    {
      kind: "toggle",
      section: "editor",
      label: "Line numbers",
      hint: "show the gutter",
      key: "line_numbers",
    },
    {
      kind: "choice",
      section: "startup",
      label: "Open in view",
      key: "default_view",
      options: [
        { value: "edit", label: "edit" },
        { value: "split", label: "split" },
        { value: "preview", label: "preview" },
      ],
    },
    {
      kind: "choice",
      section: "startup",
      label: "Window buttons",
      key: "window_controls",
      options: [
        { value: "auto", label: "auto (hide on tiling WMs)" },
        { value: "always", label: "always" },
        { value: "never", label: "never" },
      ],
    },
    {
      kind: "choice",
      section: "startup",
      label: "Rounded corners",
      key: "rounded_corners",
      options: [
        { value: "auto", label: "auto (not on tiling WMs)" },
        { value: "always", label: "always" },
        { value: "never", label: "never" },
      ],
    },
    ...ACTIONS.map((action) => ({ kind: "key", section: "keybindings", action }) as Row),
  ] as Row[];

  let sel = 0;
  let panel: HTMLDivElement;
  let fontInput: HTMLInputElement;

  function set<K extends keyof Prefs>(key: K, value: Prefs[K]) {
    prefs[key] = value;
  }

  function currentIndex(row: Extract<Row, { kind: "choice" }>): number {
    const i = row.options.findIndex((o) => o.value === prefs[row.key]);
    return i === -1 ? 0 : i;
  }

  // `p` is only here so the template re-evaluates when prefs change.
  function labelOf(row: Extract<Row, { kind: "choice" }>, p: Prefs): string {
    const i = row.options.findIndex((o) => o.value === p[row.key]);
    return row.options[i === -1 ? 0 : i].label;
  }

  function cycle(row: Extract<Row, { kind: "choice" }>, dir: number) {
    const n = row.options.length;
    const next = row.options[(currentIndex(row) + dir + n) % n];
    set(row.key, next.value as never);
  }

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

  // ----- keyboard navigation -----
  function activate(row: Row) {
    if (row.kind === "choice") cycle(row, 1);
    else if (row.kind === "toggle") set(row.key, !prefs[row.key]);
    else if (row.kind === "text") fontInput?.focus();
    else startRecording(row.action.id);
  }

  function move(delta: number) {
    sel = Math.min(rows.length - 1, Math.max(0, sel + delta));
    tick().then(() =>
      panel?.querySelector(".srow.sel")?.scrollIntoView({ block: "nearest" }),
    );
  }

  function onKey(e: KeyboardEvent) {
    if (!open || recording) return;
    if (e.ctrlKey || e.metaKey || e.altKey) return; // app shortcuts keep working

    const target = e.target as HTMLElement;
    if (target instanceof HTMLInputElement) {
      if (e.key === "Enter" || e.key === "Escape") {
        e.preventDefault();
        e.stopImmediatePropagation();
        target.blur();
      }
      return;
    }

    const row = rows[sel];
    let handled = true;
    switch (e.key) {
      case "ArrowDown":
      case "j":
        move(1);
        break;
      case "ArrowUp":
      case "k":
        move(-1);
        break;
      case "Home":
        move(-rows.length);
        break;
      case "End":
        move(rows.length);
        break;
      case "ArrowRight":
      case "l":
        if (row.kind === "choice") cycle(row, 1);
        else if (row.kind === "toggle") set(row.key, !prefs[row.key]);
        break;
      case "ArrowLeft":
      case "h":
        if (row.kind === "choice") cycle(row, -1);
        else if (row.kind === "toggle") set(row.key, !prefs[row.key]);
        break;
      case "Enter":
      case " ":
        activate(row);
        break;
      case "Backspace":
      case "Delete":
        if (row.kind === "key") setBinding(row.action.id, null);
        break;
      case "Escape":
        open = false;
        break;
      default:
        handled = false;
    }
    if (handled) {
      e.preventDefault();
      e.stopImmediatePropagation();
    }
  }

  // Listen only while open, in the capture phase, so the editor underneath never sees
  // the arrow keys. Focus leaves the editor so typing can't leak into the document.
  let listening = false;
  $: if (open && !listening) {
    listening = true;
    (document.activeElement as HTMLElement | null)?.blur();
    window.addEventListener("keydown", onKey, true);
  } else if (!open && listening) {
    listening = false;
    window.removeEventListener("keydown", onKey, true);
    stopRecording();
  }

  function handleScrimClick(e: MouseEvent) {
    if (e.target === e.currentTarget) open = false;
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="scrim" onclick={handleScrimClick}>
    <div class="panel" role="dialog" aria-label="Settings" bind:this={panel}>
      <header>
        <span class="title">settings</span>
        <span class="esc">esc to close</span>
      </header>

      <div class="rows">
        {#each rows as row, i}
          {#if i === 0 || rows[i - 1].section !== row.section}
            <div class="shead">{row.section}</div>
          {/if}

          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="srow" class:sel={i === sel} onclick={() => (sel = i)}>
            {#if row.kind === "choice"}
              <span class="label">{row.label}</span>
              <span class="value">
                <button class="arrow" tabindex="-1" onclick={() => cycle(row, -1)}>‹</button>
                <span class="val">{labelOf(row, prefs)}</span>
                <button class="arrow" tabindex="-1" onclick={() => cycle(row, 1)}>›</button>
              </span>
            {:else if row.kind === "toggle"}
              <span class="label">
                {row.label}<small>{row.hint}</small>
              </span>
              <span class="value">
                <button
                  class="check"
                  tabindex="-1"
                  aria-pressed={!!prefs[row.key]}
                  onclick={() => set(row.key, !prefs[row.key])}
                >
                  {prefs[row.key] ? "[x]" : "[ ]"}
                </button>
              </span>
            {:else if row.kind === "text"}
              <span class="label">{row.label}</span>
              <span class="value">
                <input
                  bind:this={fontInput}
                  type="text"
                  spellcheck="false"
                  placeholder="JetBrains Mono"
                  value={prefs.font_family ?? ""}
                  onfocus={() => (sel = i)}
                  onchange={(e) =>
                    set("font_family", e.currentTarget.value.trim() || "JetBrains Mono")}
                />
              </span>
            {:else}
              <span class="label">{row.action.label}</span>
              <span class="value">
                <button
                  class="keys-btn"
                  class:recording={recording === row.action.id}
                  class:custom={!!prefs.keybindings?.[row.action.id]}
                  tabindex="-1"
                  onclick={() =>
                    recording === row.action.id
                      ? stopRecording()
                      : startRecording(row.action.id)}
                  title="Change shortcut"
                >
                  {#if recording === row.action.id}
                    <span class="press">press keys…</span>
                  {:else}
                    {#each bindings[row.action.id] as combo, n}
                      {#if n > 0}<span class="or">or</span>{/if}
                      <span class="combo">
                        {#each comboParts(combo) as part, pi}{#if pi > 0}<span class="plus">+</span>{/if}<kbd>{part}</kbd>{/each}
                      </span>
                    {/each}
                  {/if}
                </button>
                <button
                  class="reset"
                  tabindex="-1"
                  aria-label="Reset to default"
                  title="Reset to default"
                  disabled={!prefs.keybindings?.[row.action.id]}
                  onclick={() => setBinding(row.action.id, null)}
                >
                  ↺
                </button>
              </span>
            {/if}
          </div>
        {/each}
      </div>

      {#if recordError}
        <div class="error" role="alert">{recordError}</div>
      {/if}

      <footer>
        <span><kbd>↑</kbd><kbd>↓</kbd> move</span>
        <span><kbd>←</kbd><kbd>→</kbd> change</span>
        <span><kbd>enter</kbd> edit</span>
        <span><kbd>⌫</kbd> reset key</span>
        <span class="path">~/.config/cedilha/config.toml</span>
      </footer>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 8vh;
    z-index: 100;
  }

  .panel {
    width: min(580px, 94vw);
    max-height: 82vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: var(--text-min);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border);
  }

  .title {
    color: var(--accent);
    font-weight: 700;
  }

  .esc {
    color: var(--fg-dim);
  }

  .rows {
    overflow-y: auto;
    padding: 4px 0 8px;
  }

  .shead {
    padding: 10px 14px 4px;
    color: var(--fg-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: var(--text-min);
  }

  .srow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 4px 14px 4px 12px;
    border-left: 2px solid transparent;
    min-height: 28px;
  }

  .srow.sel {
    background: var(--bg-surface);
    border-left-color: var(--accent);
  }

  .label {
    color: var(--fg);
  }

  .label small {
    margin-left: 10px;
    color: var(--fg-dim);
    font-size: var(--text-min);
  }

  .value {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .val {
    min-width: 120px;
    text-align: center;
    color: var(--accent);
  }

  /* Plain text controls: no boxes, no gradients. */
  button {
    background: transparent;
    border: none;
    border-radius: 0;
    padding: 0 4px;
    color: var(--fg-dim);
    cursor: pointer;
    font: inherit;
  }

  button:hover:not(:disabled) {
    color: var(--accent);
  }

  .arrow {
    font-size: 15px;
    line-height: 1;
  }

  .check {
    color: var(--accent);
  }

  input[type="text"] {
    width: 180px;
    background: transparent;
    color: var(--accent);
    border: none;
    border-bottom: 1px solid var(--border);
    border-radius: 0;
    padding: 2px 2px;
    text-align: right;
    font: inherit;
    outline: none;
  }

  input[type="text"]:focus {
    border-bottom-color: var(--accent);
  }

  .keys-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 4px;
  }

  .combo {
    display: inline-flex;
    align-items: center;
  }

  .plus {
    color: var(--fg-dim);
    opacity: 0.5;
    margin: 0 2px;
  }

  kbd {
    font-family: var(--font-mono);
    font-size: var(--text-min);
    color: var(--fg);
  }

  .keys-btn.custom kbd {
    color: var(--accent);
  }

  .keys-btn.recording {
    outline: 1px solid var(--accent);
  }

  .press {
    color: var(--accent);
  }

  .or {
    color: var(--fg-dim);
  }

  .reset {
    font-size: 14px;
  }

  .reset:disabled {
    opacity: 0.2;
    cursor: default;
  }

  .error {
    margin: 0 14px 8px;
    padding: 4px 8px;
    color: var(--danger);
    border: 1px solid var(--danger);
    border-radius: var(--radius);
  }

  footer {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 16px;
    padding: 8px 14px;
    border-top: 1px solid var(--border);
    color: var(--fg-dim);
  }

  footer kbd {
    color: var(--fg-dim);
  }

  footer .path {
    margin-left: auto;
    opacity: 0.7;
  }
</style>
