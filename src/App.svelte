<script lang="ts">
  import { onMount } from "svelte";
  import Editor from "./components/Editor.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import Settings from "./components/Settings.svelte";
  import * as api from "./lib/tauri";
  import type { Prefs } from "./lib/tauri";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
  try {
    appWindow = getCurrentWindow();
  } catch {
    // Running in browser dev mode outside Tauri webview
  }

  // ----- state -----
  let content = "# Welcome to Cedilha\n\nCedilha is an opinionated, distraction-free **Markdown & LaTeX** editor built with Rust and Tauri for the Omarchy aesthetic.\n\n### Math Formulas\n\nInline math works directly: $E = mc^2$, or try the Pythagorean theorem: $a^2 + b^2 = c^2$.\n\nYou can also write centered block formulas:\n\n$$\nf(x) = \\int_{-\\infty}^{\\infty} \\hat{f}(\\xi)\\,e^{2 \\pi i \\xi x}\\,d\\xi\n$$\n\nClick on any math formula above to edit its raw LaTeX, and click away to see it render live!\n";
  let path: string | null = null;
  let dirty = false;
  let prefs: Prefs = {
    theme: "catppuccin-mocha",
    font_size: 15,
    tab_size: 2,
    wrap: true,
    typewriter: true,
    line_numbers: false,
  };
  let settingsOpen = false;
  let editor: InstanceType<typeof Editor>;
  let stats = { words: 0, chars: 0, lines: 1 };
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  // ----- lifecycle -----
  onMount(() => {
    api.getPrefs().then((p) => applyPrefs(p)).catch(() => {});

    // Compute initial stats
    computeStats(content);

    // Load initial file argument if passed via CLI: `cedilha document.md`
    const params = new URLSearchParams(window.location.search);
    const initial = params.get("file");
    if (initial) {
      api.readDocument(initial).then((doc) => loadInto(doc, initial)).catch(() => {});
    }
  });

  function applyPrefs(p: Prefs) {
    prefs = { ...p };
    document.documentElement.setAttribute("data-theme", p.theme);
    document.documentElement.style.setProperty("--editor-font-size", `${p.font_size}px`);
    document.documentElement.style.setProperty("--editor-tab-size", `${p.tab_size}`);
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
    } else if (mod && (e.key === "," || e.key === "p" && e.shiftKey)) {
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

  // Window control helpers
  function minimizeWindow() {
    appWindow?.minimize();
  }

  function toggleMaximize() {
    appWindow?.toggleMaximize();
  }

  function closeWindow() {
    appWindow?.close();
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="app" data-theme={prefs.theme}>
  <header class="toolbar" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <span class="logo">ç</span>
      <span class="name">cedilha</span>
    </div>

    <nav class="actions">
      <button onclick={newDoc} title="New Document (Ctrl+N)">New <kbd>Ctrl+N</kbd></button>
      <button onclick={openFile} title="Open Document (Ctrl+O)">Open <kbd>Ctrl+O</kbd></button>
      <button class="primary" onclick={save} title="Save (Ctrl+S)">
        Save {#if dirty}* {/if}<kbd>Ctrl+S</kbd>
      </button>
      <button onclick={() => (settingsOpen = !settingsOpen)} title="Settings (Ctrl+,)">
        ⚙ Settings
      </button>
    </nav>

    {#if appWindow}
      <div class="window-controls">
        <button class="win-btn" onclick={minimizeWindow} aria-label="Minimize">─</button>
        <button class="win-btn" onclick={toggleMaximize} aria-label="Maximize">□</button>
        <button class="win-btn close" onclick={closeWindow} aria-label="Close">✕</button>
      </div>
    {/if}
  </header>

  <main class="workspace">
    <Editor
      bind:this={editor}
      content={content}
      fontSize={prefs.font_size}
      tabSize={prefs.tab_size}
      wrap={prefs.wrap}
      typewriter={prefs.typewriter}
      lineNumbers={prefs.line_numbers}
      on:change={handleEditorChange}
    />
  </main>

  <StatusBar
    path={path}
    dirty={dirty}
    wordCount={stats.words}
    charCount={stats.chars}
    lineCount={stats.lines}
  />

  <Settings bind:open={settingsOpen} bind:prefs={prefs} />
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
    height: 42px;
    padding: 0 12px;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
    flex: 0 0 auto;
    cursor: default;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
    color: var(--accent);
    font-size: 15px;
    letter-spacing: 0.5px;
  }

  .logo {
    font-size: 20px;
    line-height: 1;
    font-family: serif;
    font-weight: bold;
    color: var(--accent);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .actions button {
    font-size: 12px;
    padding: 4px 10px;
    display: inline-flex;
    align-items: center;
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 9px;
    opacity: 0.7;
    margin-left: 6px;
    padding: 1px 4px;
    background: var(--bg-surface);
    border-radius: 3px;
    border: 1px solid var(--border);
  }

  .window-controls {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .win-btn {
    background: transparent;
    border: none;
    padding: 4px 8px;
    font-size: 11px;
    color: var(--fg-dim);
    border-radius: 4px;
  }

  .win-btn:hover {
    background: var(--bg-surface);
    color: var(--fg);
  }

  .win-btn.close:hover {
    background: var(--danger);
    color: var(--bg);
  }

  .workspace {
    flex: 1 1 auto;
    overflow: hidden;
    background: var(--bg);
    display: flex;
    flex-direction: column;
    user-select: text;
  }
</style>
