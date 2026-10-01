<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import Editor from "./components/Editor.svelte";
  import Preview from "./components/Preview.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import { resolveBindings, parseCombo, matches, type ActionId } from "./lib/keybindings";
  import Settings from "./components/Settings.svelte";
  import * as api from "./lib/tauri";
  import type { Prefs, OmarchyTheme } from "./lib/tauri";

  type ViewMode = "edit" | "split" | "preview";

  // ----- state -----
  let content = "# Welcome to Cedilha\n\nCedilha is an opinionated, distraction-free **Markdown & LaTeX** editor built with Rust and Tauri for the Omarchy aesthetic.\n\n### Math Formulas\n\nInline math works directly: $E = mc^2$, or try the Pythagorean theorem: $a^2 + b^2 = c^2$.\n\nYou can also write centered block formulas:\n\n$$\nf(x) = \\int_{-\\infty}^{\\infty} \\hat{f}(\\xi)\\,e^{2 \\pi i \\xi x}\\,d\\xi\n$$\n\nToggle **Split view** in the top bar (or press `Ctrl+P`) to see live rendered HTML and KaTeX side-by-side!\n";
  let path: string | null = null;
  let dirty = false;
  let viewMode: ViewMode = "edit";
  let prefs: Prefs = {
    theme: "omarchy",
    font_size: 15,
    tab_size: 2,
    typewriter: true,
    line_numbers: true,
  };
  let omarchyTheme: OmarchyTheme | null = null;
  let settingsOpen = false;
  let editor: InstanceType<typeof Editor>;
  let preview: InstanceType<typeof Preview>;
  let stats = { words: 0, chars: 0, lines: 1 };
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  // Two-way scroll sync. Whichever pane the user is scrolling owns the sync; scroll events
  // caused by the other pane being moved programmatically are ignored until things settle.
  let syncOwner: "editor" | "preview" | null = null;
  let syncTimer: ReturnType<typeof setTimeout> | null = null;
  let syncRaf: number | null = null;

  function syncFrom(source: "editor" | "preview") {
    if (viewMode !== "split" || !editor || !preview) return;
    if (syncTimer) clearTimeout(syncTimer);
    syncTimer = setTimeout(() => {
      syncOwner = null;
      syncTimer = null;
    }, 150);
    if (syncOwner && syncOwner !== source) return;
    syncOwner = source;
    if (syncRaf !== null) cancelAnimationFrame(syncRaf);
    syncRaf = requestAnimationFrame(() => {
      syncRaf = null;
      if (source === "editor") {
        preview?.scrollToAnchor(editor.getScrollAnchor());
      } else {
        editor?.scrollToAnchor(preview.getScrollAnchor());
      }
    });
  }

  function injectOmarchyTheme(theme: OmarchyTheme) {
    omarchyTheme = theme;
    let styleEl = document.getElementById("omarchy-theme-style");
    if (!styleEl) {
      styleEl = document.createElement("style");
      styleEl.id = "omarchy-theme-style";
      document.head.appendChild(styleEl);
    }
    styleEl.textContent = `
      :root[data-theme="omarchy"] {
        --bg: ${theme.bg};
        --bg-elev: ${theme.bg_elev};
        --bg-surface: ${theme.bg_surface};
        --fg: ${theme.fg};
        --fg-dim: ${theme.fg_dim};
        --accent: ${theme.accent};
        --accent-2: ${theme.accent2};
        --danger: ${theme.danger};
        --success: ${theme.success};
        --warning: ${theme.warning};
        --border: ${theme.border};
        --selection: ${theme.selection};
        --marker: ${theme.marker};
        --active-line: ${theme.active_line};
      }
    `;
  }

  // ----- lifecycle -----
  onMount(() => {
    let unlisten: UnlistenFn | undefined;

    api.getPrefs().then((p) => applyPrefs(p)).catch(() => {});

    api.getOmarchyTheme().then((theme) => {
      if (theme) {
        injectOmarchyTheme(theme);
      }
    }).catch(() => {});

    listen<OmarchyTheme>("omarchy-theme-changed", (event) => {
      if (event.payload) {
        injectOmarchyTheme(event.payload);
      }
    }).then((fn) => {
      unlisten = fn;
    }).catch(() => {});

    const onFocus = () => {
      api.getOmarchyTheme().then((theme) => {
        if (theme) {
          injectOmarchyTheme(theme);
        }
      }).catch(() => {});
    };
    window.addEventListener("focus", onFocus);

    let unlistenConfig: UnlistenFn | undefined;
    let unlistenClose: UnlistenFn | undefined;

    listen("close-requested", async () => {
      try {
        if (await confirmDiscard()) {
          await api.exitApp();
        } else {
          await api.cancelClose();
        }
      } catch {
        api.cancelClose().catch(() => {});
      }
    }).then((fn) => {
      unlistenClose = fn;
    }).catch(() => {});

    listen<Prefs>("config-changed", (event) => {
      if (event.payload) {
        applyPrefs(event.payload);
      }
    }).then((fn) => {
      unlistenConfig = fn;
    }).catch(() => {});

    // Compute initial stats
    scheduleComputeStats(content, true);

    // Load initial file argument if passed via CLI: `cedilha document.md`
    const params = new URLSearchParams(window.location.search);
    const initial = params.get("file");
    if (initial) {
      api.readDocument(initial).then((doc) => loadInto(doc, initial)).catch(() => {});
    }

    return () => {
      window.removeEventListener("focus", onFocus);
      if (unlisten) unlisten();
      if (unlistenConfig) unlistenConfig();
      if (unlistenClose) unlistenClose();
    };
  });

  // JSON of the prefs last loaded from / written to the backend. Empty until the first load,
  // so the built-in defaults above can never be saved over the user's config.
  let prefsSynced = "";

  function applyPrefs(p: Prefs) {
    prefs = { ...p };
    prefsSynced = JSON.stringify(prefs);
    applyPrefsStyles(p);
  }

  function applyPrefsStyles(p: Prefs) {
    document.documentElement.setAttribute("data-theme", p.theme);
    document.documentElement.style.setProperty("--editor-font-size", `${p.font_size}px`);
    document.documentElement.style.setProperty("--editor-tab-size", `${p.tab_size}`);
    if (p.font_family) {
      document.documentElement.style.setProperty(
        "--font-mono",
        `"${p.font_family}", "JetBrains Mono", monospace`,
      );
    }
    if (p.editor_max_width) {
      document.documentElement.style.setProperty("--editor-max-width", `${p.editor_max_width}px`);
    }
  }

  function loadInto(doc: string, p: string | null) {
    // CodeMirror normalizes line endings to \n; keep the rendered text identical so
    // preview source offsets line up with the editor document.
    doc = doc.replace(/\r\n?/g, "\n");
    content = doc;
    path = p;
    dirty = false;
    autosavePaused = false;
    scheduleComputeStats(doc, true);
    if (editor) {
      editor.setContent(doc);
    }
  }

  // ----- file operations -----
  let notice = "";
  let noticeTimer: ReturnType<typeof setTimeout> | null = null;
  let autosavePaused = false;

  function showNotice(msg: string, ms = 8000) {
    notice = msg;
    if (noticeTimer) clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => (notice = ""), ms);
  }

  function errorText(e: unknown): string {
    return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
  }

  // Resolves true when it is fine to replace the current document.
  async function confirmDiscard(): Promise<boolean> {
    if (!dirty) return true;
    const name = path ? path.split(/[/\\]/).pop() || path : "Untitled";
    const choice = await api.askUnsaved(name);
    if (choice === "save") {
      await save();
      return !dirty;
    }
    return choice === "discard";
  }

  async function openFile() {
    try {
      if (!(await confirmDiscard())) return;
      const res = await api.openFile();
      if (res.path) {
        loadInto(res.content, res.path);
      }
    } catch (e) {
      console.error("Failed to open file:", e);
      showNotice(`Open failed: ${errorText(e)}`);
    }
  }

  async function newDoc() {
    try {
      if (!(await confirmDiscard())) return;
      const doc = await api.newDocument();
      loadInto(doc, null);
    } catch {
      loadInto("", null);
    }
  }

  async function save() {
    const snapshot = content;
    try {
      if (!path) {
        const p = await api.saveAsFile(snapshot);
        if (p) {
          path = p;
          dirty = content !== snapshot;
          autosavePaused = false;
        }
        return;
      }
      await api.saveDocument(path, snapshot, true);
      // Edits typed while the write was in flight are not saved yet.
      dirty = content !== snapshot;
      autosavePaused = false;
    } catch (e) {
      console.error("Failed to save:", e);
      showNotice(`Save failed: ${errorText(e)}`);
    }
  }

  async function saveAs() {
    const snapshot = content;
    try {
      const p = await api.saveAsFile(snapshot);
      if (p) {
        path = p;
        dirty = content !== snapshot;
        autosavePaused = false;
      }
    } catch (e) {
      console.error("Failed to save as:", e);
      showNotice(`Save failed: ${errorText(e)}`);
    }
  }

  let statsTimer: ReturnType<typeof setTimeout> | null = null;

  function computeStatsFast(text: string) {
    const chars = text.length;
    let lines = text.length > 0 ? 1 : 0;
    let words = 0;
    let inWord = false;

    for (let i = 0; i < text.length; i++) {
      const code = text.charCodeAt(i);
      if (code === 10) {
        lines++;
      }
      if (code <= 32 && (code === 32 || (code >= 9 && code <= 13))) {
        inWord = false;
      } else if (!inWord) {
        inWord = true;
        words++;
      }
    }

    stats = { words, chars, lines: lines || 1 };
  }

  function scheduleComputeStats(text: string, immediate = false) {
    if (statsTimer) clearTimeout(statsTimer);
    if (immediate) {
      computeStatsFast(text);
    } else {
      statsTimer = setTimeout(() => {
        computeStatsFast(text);
      }, 150);
    }
  }

  function handleEditorChange(e: CustomEvent<string>) {
    content = e.detail;
    dirty = true;
    scheduleComputeStats(content, false);
    queueAutosave();
  }

  function queueAutosave() {
    if (!dirty || !path || autosavePaused) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      if (!path || !dirty || autosavePaused) return;
      const snapshot = content;
      try {
        await api.saveDocument(path, snapshot, false);
        dirty = content !== snapshot;
      } catch (e) {
        const msg = errorText(e);
        if (msg.startsWith("CONFLICT")) {
          autosavePaused = true;
          showNotice(`File changed on disk: autosave paused (${hint("save")} overwrites it)`, 15000);
        } else {
          showNotice(`Autosave failed: ${msg}`);
        }
      }
    }, 1500);
  }

  // ----- keyboard shortcuts -----
  $: bindings = resolveBindings(prefs.keybindings);

  const keyActions: Partial<Record<ActionId, () => void>> = {
    save: () => void save(),
    save_as: () => void saveAs(),
    open: () => void openFile(),
    new: () => void newDoc(),
    find: () => {
      if (viewMode === "preview") viewMode = "split";
      editor?.openSearch();
    },
    cycle_view: () => {
      viewMode = viewMode === "edit" ? "split" : viewMode === "split" ? "preview" : "edit";
    },
    settings: () => {
      settingsOpen = !settingsOpen;
    },
  };

  function onKeydown(e: KeyboardEvent) {
    if (e.defaultPrevented) return;
    for (const id of Object.keys(keyActions) as ActionId[]) {
      const hit = bindings[id].some((c) => {
        const combo = parseCombo(c);
        return combo !== null && matches(e, combo);
      });
      if (hit) {
        e.preventDefault();
        keyActions[id]!();
        return;
      }
    }
  }

  // Short label for tooltips, e.g. "Ctrl+Shift+S" (first binding of an action).
  $: hint = (id: ActionId) => bindings[id][0];

  // Persist preference updates
  function onPrefsUpdated() {
    prefsSynced = JSON.stringify(prefs);
    applyPrefsStyles(prefs);
    api.savePrefs(prefs).catch(() => {});
  }

  // Only genuine changes (Settings panel) are persisted, never the initial defaults.
  $: if (prefsSynced && JSON.stringify(prefs) !== prefsSynced) {
    onPrefsUpdated();
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="app" data-theme={prefs.theme}>
  <header class="toolbar" data-tauri-drag-region>
    <!-- Left: Logo & Document Operations -->
    <div class="toolbar-left" data-tauri-drag-region>
      <div class="brand" data-tauri-drag-region title="Cedilha">
        <span class="logo">ç</span>
      </div>

      <nav class="doc-actions">
        <button onclick={newDoc} title="New Document ({hint('new')})">New</button>
        <button onclick={openFile} title="Open Document ({hint('open')})">Open</button>
        <button class="save-btn" class:dirty onclick={save} title="Save ({hint('save')})">
          Save{#if dirty}<span class="dirty-dot">●</span>{/if}
        </button>
      </nav>
    </div>

    <!-- Center: Segmented View Switcher (Apostrophe style) -->
    <div class="toolbar-center" data-tauri-drag-region>
      <div class="view-switcher" role="radiogroup" aria-label="View mode">
        <button
          class:active={viewMode === "edit"}
          onclick={() => (viewMode = "edit")}
          title="Code only"
        >
          Edit
        </button>
        <button
          class:active={viewMode === "split"}
          onclick={() => (viewMode = "split")}
          title="Side-by-side split"
        >
          Split
        </button>
        <button
          class:active={viewMode === "preview"}
          onclick={() => (viewMode = "preview")}
          title="Rendered preview only ({hint('cycle_view')} cycles views)"
        >
          Preview
        </button>
      </div>
    </div>

    <!-- Right: Settings -->
    <div class="toolbar-right" data-tauri-drag-region>
      <div class="util-actions">
        <button
          class="settings-btn"
          onclick={() => (settingsOpen = !settingsOpen)}
          title="Settings ({hint('settings')})"
        >
          <span>Settings</span>
          <kbd class="key-badge">{hint('settings').replace(/\+/g, " ")}</kbd>
        </button>
      </div>
    </div>
  </header>

  <main class="workspace" class:split={viewMode === "split"}>
    <div
      class="pane editor-pane"
      class:half={viewMode === "split"}
      class:hidden={viewMode === "preview"}
    >
      <Editor
        bind:this={editor}
        content={content}
        fontSize={prefs.font_size}
        tabSize={prefs.tab_size}
        typewriter={prefs.typewriter}
        lineNumbers={prefs.line_numbers}
        findNextKeys={bindings.find_next}
        findPrevKeys={bindings.find_prev}
        on:change={handleEditorChange}
        on:scroll={() => syncFrom("editor")}
      />
    </div>

    {#if viewMode === "split"}
      <div class="split-divider"></div>
    {/if}

    <div
      class="pane preview-pane"
      class:half={viewMode === "split"}
      class:hidden={viewMode === "edit"}
    >
      <Preview
        bind:this={preview}
        {content}
        centered={viewMode === "preview"}
        active={viewMode !== "edit"}
        on:scroll={() => syncFrom("preview")}
      />
    </div>
  </main>

  <StatusBar
    path={path}
    dirty={dirty}
    wordCount={stats.words}
    charCount={stats.chars}
    lineCount={stats.lines}
    {notice}
  />

  <Settings bind:open={settingsOpen} bind:prefs={prefs} {omarchyTheme} />
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: var(--bg);
    color: var(--fg);
    user-select: none;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 40px;
    padding: 0 12px;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
    flex: 0 0 auto;
    cursor: default;
    user-select: none;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1 1 0;
  }

  .toolbar-center {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    flex: 1 1 0;
  }

  .brand {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .logo {
    font-size: 22px;
    line-height: 1;
    font-family: serif;
    font-weight: 700;
    color: var(--accent);
    user-select: none;
    padding: 0 4px;
    cursor: default;
    /* Optical correction for the cedilla descender hook */
    transform: translateY(-3px);
  }

  .doc-actions,
  .util-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .toolbar button {
    background: transparent;
    border: 1px solid transparent;
    font-size: 12px;
    font-weight: 500;
    padding: 4px 10px;
    color: var(--fg);
    border-radius: var(--radius);
    transition: all 0.12s ease;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .toolbar button:hover {
    background: var(--bg-surface);
    border-color: var(--border);
  }

  .settings-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
  }

  .key-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    line-height: 1;
    padding: 2px 5px;
    color: var(--fg-dim);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 3px;
    letter-spacing: -0.01em;
    pointer-events: none;
    transition: color 0.12s ease, border-color 0.12s ease;
  }

  .settings-btn:hover .key-badge {
    color: var(--accent);
    border-color: var(--accent);
  }

  .save-btn.dirty {
    color: var(--warning);
    font-weight: 600;
  }

  .dirty-dot {
    font-size: 8px;
    color: var(--warning);
    margin-left: 2px;
  }

  /* Segmented View Switcher (Apostrophe style) */
  .view-switcher {
    display: flex;
    background: var(--bg-surface);
    border-radius: var(--radius);
    padding: 2px;
    border: 1px solid var(--border);
    gap: 2px;
  }

  .view-switcher button {
    background: transparent;
    border: none;
    font-size: 11px;
    font-weight: 500;
    padding: 3px 12px;
    color: var(--fg-dim);
    border-radius: calc(var(--radius) - 2px);
    transition: all 0.12s ease;
    cursor: pointer;
  }

  .view-switcher button:hover {
    color: var(--fg);
  }

  .view-switcher button.active {
    background: var(--accent);
    color: var(--bg);
    font-weight: 700;
  }

  .workspace {
    flex: 1 1 auto;
    overflow: hidden;
    background: var(--bg);
    display: flex;
    flex-direction: row;
    position: relative;
  }

  .pane {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .pane.hidden {
    display: none !important;
  }

  .pane.editor-pane {
    flex: 1 1 auto;
    width: 100%;
  }

  .pane.preview-pane {
    flex: 1 1 auto;
    width: 100%;
  }

  .pane.half {
    flex: 1 1 50%;
    width: 50%;
  }

  .pane.editor-pane.half {
    border-right: 1px solid var(--border);
  }

  .split-divider {
    display: none;
  }
</style>
