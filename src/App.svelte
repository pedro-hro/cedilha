<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import Editor from "./components/Editor.svelte";
  import Preview from "./components/Preview.svelte";
  import StatusBar from "./components/StatusBar.svelte";
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
    line_numbers: false,
  };
  let omarchyTheme: OmarchyTheme | null = null;
  let settingsOpen = false;
  let editor: InstanceType<typeof Editor>;
  let stats = { words: 0, chars: 0, lines: 1 };
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

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

    listen<Prefs>("config-changed", (event) => {
      if (event.payload) {
        applyPrefs(event.payload);
      }
    }).then((fn) => {
      unlistenConfig = fn;
    }).catch(() => {});

    // Compute initial stats
    computeStats(content);

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
    };
  });

  function applyPrefs(p: Prefs) {
    prefs = { ...p };
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
    content = doc;
    path = p;
    dirty = false;
    computeStats(doc);
    if (editor) {
      editor.setContent(doc);
    }
  }

  // ----- file operations -----
  async function openFile() {
    try {
      const res = await api.openFile();
      if (res.path) {
        loadInto(res.content, res.path);
      }
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  }

  async function newDoc() {
    try {
      const doc = await api.newDocument();
      loadInto(doc, null);
    } catch {
      loadInto("", null);
    }
  }

  async function save() {
    try {
      if (!path) {
        const p = await api.saveAsFile(content);
        if (p) {
          path = p;
          dirty = false;
        }
        return;
      }
      await api.saveDocument(path, content);
      dirty = false;
    } catch (e) {
      console.error("Failed to save:", e);
    }
  }

  async function saveAs() {
    try {
      const p = await api.saveAsFile(content);
      if (p) {
        path = p;
        dirty = false;
      }
    } catch (e) {
      console.error("Failed to save as:", e);
    }
  }

  function handleEditorChange(e: CustomEvent<string>) {
    content = e.detail;
    dirty = true;
    computeStats(content);
    queueAutosave();
  }

  function queueAutosave() {
    if (!dirty || !path) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      if (path && dirty) {
        api.saveDocument(path, content).then(() => (dirty = false)).catch(() => {});
      }
    }, 1500);
  }

  // ----- stats -----
  function computeStats(text: string) {
    const chars = text.length;
    const lines = text ? text.split("\n").length : 1;
    const words = (text.match(/\S+/g) ?? []).length;
    stats = { words, chars, lines };
  }

  // ----- keyboard shortcuts -----
  function onKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key.toLowerCase() === "s") {
      e.preventDefault();
      if (e.shiftKey) {
        saveAs();
      } else {
        save();
      }
    } else if (mod && e.key.toLowerCase() === "o") {
      e.preventDefault();
      openFile();
    } else if (mod && e.key.toLowerCase() === "n") {
      e.preventDefault();
      newDoc();
    } else if (mod && e.key === "1") {
      e.preventDefault();
      viewMode = "edit";
    } else if (mod && e.key === "2") {
      e.preventDefault();
      viewMode = "split";
    } else if (mod && e.key === "3") {
      e.preventDefault();
      viewMode = "preview";
    } else if (mod && e.key.toLowerCase() === "p" && !e.shiftKey) {
      e.preventDefault();
      viewMode = viewMode === "edit" ? "split" : viewMode === "split" ? "preview" : "edit";
    } else if (mod && (e.key === "," || (e.key === "p" && e.shiftKey))) {
      e.preventDefault();
      settingsOpen = !settingsOpen;
    }
  }

  // Persist preference updates
  function onPrefsUpdated() {
    applyPrefs(prefs);
    api.savePrefs(prefs).catch(() => {});
  }

  $: if (prefs) {
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
        <button onclick={newDoc} title="New Document (Ctrl+N)">New</button>
        <button onclick={openFile} title="Open Document (Ctrl+O)">Open</button>
        <button class="save-btn" class:dirty onclick={save} title="Save (Ctrl+S)">
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
          title="Code only (Ctrl+1)"
        >
          Edit
        </button>
        <button
          class:active={viewMode === "split"}
          onclick={() => (viewMode = "split")}
          title="Side-by-side split (Ctrl+2)"
        >
          Split
        </button>
        <button
          class:active={viewMode === "preview"}
          onclick={() => (viewMode = "preview")}
          title="Rendered preview only (Ctrl+3)"
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
          title="Settings (Ctrl+,)"
        >
          <span>Settings</span>
          <kbd class="key-badge">Ctrl ,</kbd>
        </button>
      </div>
    </div>
  </header>

  <main class="workspace" class:split={viewMode === "split"}>
    {#if viewMode === "edit" || viewMode === "split"}
      <div class="pane editor-pane" class:half={viewMode === "split"}>
        <Editor
          bind:this={editor}
          content={content}
          fontSize={prefs.font_size}
          tabSize={prefs.tab_size}
          typewriter={prefs.typewriter}
          lineNumbers={prefs.line_numbers}
          on:change={handleEditorChange}
        />
      </div>
    {/if}

    {#if viewMode === "split"}
      <div class="split-divider"></div>
    {/if}

    {#if viewMode === "preview" || viewMode === "split"}
      <div class="pane preview-pane" class:half={viewMode === "split"}>
        <Preview {content} centered={viewMode === "preview"} />
      </div>
    {/if}
  </main>

  <StatusBar
    path={path}
    dirty={dirty}
    wordCount={stats.words}
    charCount={stats.chars}
    lineCount={stats.lines}
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
