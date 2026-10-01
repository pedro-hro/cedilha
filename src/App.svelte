<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import Editor from "./components/Editor.svelte";
  import Preview from "./components/Preview.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import UnsavedDialog from "./components/UnsavedDialog.svelte";
  import welcome from "./lib/welcome.md?raw";
  import { resolveBindings, parseCombo, matches, type ActionId } from "./lib/keybindings";
  import Settings from "./components/Settings.svelte";
  import * as api from "./lib/tauri";
  import type { Prefs, OmarchyTheme } from "./lib/tauri";

  type ViewMode = "edit" | "split" | "preview";

  // ----- state -----
  let content = "";
  let path: string | null = null;
  let dirty = false;
  let viewMode: ViewMode = "edit";
  let prefs: Prefs = {
    theme: "gruvbox-dark",
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
  let cursor = { line: 1, col: 1 };
  // Window buttons replace the title bar we don't have. Hidden by default on tiling WMs.
  let tiling = false;
  $: showControls =
    prefs.window_controls === "always" || (prefs.window_controls !== "never" && !tiling);
  // Rounded corners only where the desktop doesn't round windows for us, and never while
  // fullscreen or maximized (the corners would leave gaps at the screen edges). Tiling
  // window managers report their tiled windows as "maximized", so that flag is ignored there.
  let maximized = false;
  let fullscreen = false;
  $: rounded =
    (prefs.rounded_corners === "always" || (prefs.rounded_corners !== "never" && !tiling)) &&
    !fullscreen &&
    !(maximized && !tiling);
  $: if (typeof document !== "undefined") {
    document.documentElement.toggleAttribute("data-rounded", rounded);
  }
  $: docName = path ? path.split(/[/\\]/).pop() || path : "untitled";
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

    api.isTilingWm().then((t) => (tiling = t)).catch(() => {});

    // Track maximized / fullscreen so the corners can square off.
    const win = getCurrentWindow();
    const syncWindowState = async () => {
      try {
        maximized = await win.isMaximized();
        fullscreen = await win.isFullscreen();
      } catch {
        /* not available: keep the current state */
      }
    };
    void syncWindowState();
    let unlistenResize: UnlistenFn | undefined;
    win.onResized(syncWindowState).then((fn) => (unlistenResize = fn)).catch(() => {});

    Promise.all([api.getPrefs(), api.initialFile().catch(() => null)]).then(([p, file]) => {
      applyPrefs(p);
      // Launch view comes from settings; later config reloads must not switch views.
      if (p.default_view === "edit" || p.default_view === "split" || p.default_view === "preview") {
        viewMode = p.default_view;
      }
      if (file?.path) {
        loadInto(file.content, file.path);
      } else if (!p.welcome_seen) {
        // First launch: show the welcome page once. Saved right away through the prefs watcher.
        openWelcome();
        prefs.welcome_seen = true;
      }
    }).catch(() => {});

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
    document.addEventListener("click", onLinkClick, true);
    document.addEventListener("auxclick", onLinkClick, true);

    let unlistenConfig: UnlistenFn | undefined;
    let unlistenClose: UnlistenFn | undefined;

    listen("close-requested", () => void requestQuit()).then((fn) => {
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

    return () => {
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("click", onLinkClick, true);
      document.removeEventListener("auxclick", onLinkClick, true);
      if (unlisten) unlisten();
      if (unlistenConfig) unlistenConfig();
      if (unlistenClose) unlistenClose();
      if (unlistenResize) unlistenResize();
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

  // Virtual document path for the untitled welcome page, only used to find its pictures.
  let welcomeBase: string | null = null;

  function loadInto(doc: string, p: string | null) {
    welcomeBase = null;
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

  // Quit, asking about unsaved changes first. Used by the window's close button / the
  // window manager and by the Close keybinding alike.
  async function requestQuit() {
    try {
      if (await confirmDiscard()) {
        await api.exitApp();
      } else {
        await api.cancelClose();
      }
    } catch {
      api.cancelClose().catch(() => {});
    }
  }

  // In-app unsaved-changes prompt (see UnsavedDialog.svelte).
  let unsavedPrompt: { name: string; resolve: (c: api.UnsavedChoice) => void } | null = null;

  function askUnsaved(name: string): Promise<api.UnsavedChoice> {
    return new Promise((resolve) => {
      unsavedPrompt = {
        name,
        resolve: (c) => {
          unsavedPrompt = null;
          resolve(c);
        },
      };
    });
  }

  // Resolves true when it is fine to replace the current document.
  async function confirmDiscard(): Promise<boolean> {
    if (!dirty) return true;
    if (unsavedPrompt) return false; // already asking
    const name = path ? path.split(/[/\\]/).pop() || path : "untitled";
    const choice = await askUnsaved(name);
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

  // The welcome page always opens as a fresh untitled copy, so edits never touch the original.
  async function openWelcome() {
    loadInto(welcome, null);
    if (viewMode === "edit") viewMode = "split";
    // The page's own pictures live in the app's resource folder: borrow it as the folder to
    // resolve images against, until the copy is saved somewhere or another file replaces it.
    const dir = await api.welcomeAssetsDir().catch(() => null);
    if (dir && path === null && content === welcome) welcomeBase = `${dir}/welcome.md`;
  }

  async function showWelcome() {
    if (!(await confirmDiscard())) return;
    await openWelcome();
  }

  // Links in the preview must never navigate the app window away: in-page anchors scroll the
  // preview, web and mail links open in the system browser, anything else is ignored.
  function onLinkClick(e: MouseEvent) {
    if (e.defaultPrevented || e.button > 1) return;
    const a = (e.target as Element | null)?.closest?.("a[href]");
    if (!a) return;
    const href = a.getAttribute("href") ?? "";
    e.preventDefault();
    if (href.startsWith("#")) {
      let id = href.slice(1);
      try {
        id = decodeURIComponent(id);
      } catch {
        /* keep the raw id */
      }
      document.getElementById(id)?.scrollIntoView({ block: "start" });
    } else if (/^(https?:|mailto:)/i.test(href)) {
      api.openUrl(href).catch((err) => showNotice(`Could not open link: ${errorText(err)}`));
    }
  }

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
    minimize: () => void api.minimizeWindow(),
    maximize: () => void api.toggleMaximize(),
    close: () => void requestQuit(),
    welcome: () => void showWelcome(),
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

    // Keyboard-first: if focus fell to <body> (clicking the preview, closing a dialog...),
    // give the key to the editor instead of dropping it.
    const bare = document.activeElement === document.body || document.activeElement === null;
    const copying = (e.ctrlKey || e.metaKey) && ["c", "Insert"].includes(e.key.length === 1 ? e.key.toLowerCase() : e.key);
    const modifierOnly = ["Control", "Shift", "Alt", "Meta", "AltGraph", "CapsLock", "Dead"].includes(e.key);
    if (bare && !settingsOpen && viewMode !== "preview" && !copying && !modifierOnly) {
      if (editor?.forwardKey(e)) e.preventDefault();
    }
  }

  // A plain click in the preview shouldn't strand the keyboard: return focus to the editor.
  // (Dragging a selection keeps focus where it is so the text can still be copied.)
  function onPreviewMouseUp() {
    if (viewMode === "split" && window.getSelection()?.isCollapsed) {
      editor?.focusEditor();
    }
  }

  // Closing Settings hands the keyboard back to the editor.
  let settingsWasOpen = false;
  $: {
    if (settingsWasOpen && !settingsOpen) {
      tick().then(() => editor?.focusEditor());
    }
    settingsWasOpen = settingsOpen;
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
    <div class="toolbar-left" data-tauri-drag-region>
      <span class="logo" title="Cedilha">ç</span>
      <span class="doc-name" title={path ?? "unsaved document"} data-tauri-drag-region>
        {docName}{#if dirty}<span class="dirty-dot" title="Unsaved changes">●</span>{/if}
      </span>
    </div>

    <div class="tabs" role="tablist" aria-label="View mode">
      {#each ["edit", "split", "preview"] as mode}
        <button
          role="tab"
          aria-selected={viewMode === mode}
          class:active={viewMode === mode}
          onclick={() => (viewMode = mode as ViewMode)}
          title={mode === "edit" ? "Code only" : mode === "split" ? "Side-by-side" : "Rendered preview"}
        >
          {mode}
        </button>
      {/each}
      <span class="tab-hint" title="Cycle views">{hint("cycle_view").replace(/\+/g, " ")}</span>
    </div>

    <div class="toolbar-right" data-tauri-drag-region>
      <button
        class="settings-btn"
        onclick={() => (settingsOpen = !settingsOpen)}
        title="Settings ({hint('settings')})"
      >
        settings <span class="key-badge">{hint("settings").replace(/\+/g, " ").toLowerCase()}</span>
      </button>

      {#if showControls}
        <div class="win-controls">
          <button class="wc" aria-label="Minimize" title="Minimize ({hint('minimize')})" onclick={() => api.minimizeWindow()}>─</button>
          <button class="wc" aria-label="Maximize" title="Maximize ({hint('maximize')})" onclick={() => api.toggleMaximize()}>▢</button>
          <button class="wc close" aria-label="Close" title="Close ({hint('close')})" onclick={() => requestQuit()}>✕</button>
        </div>
      {/if}
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
        on:cursor={(e) => (cursor = e.detail)}
      />
    </div>

    {#if viewMode === "split"}
      <div class="split-divider"></div>
    {/if}

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="pane preview-pane"
      onmouseup={onPreviewMouseUp}
      class:half={viewMode === "split"}
      class:hidden={viewMode === "edit"}
    >
      <Preview
        bind:this={preview}
        {content}
        docPath={path ?? welcomeBase}
        centered={viewMode === "preview"}
        active={viewMode !== "edit"}
        on:scroll={() => syncFrom("preview")}
      />
    </div>
  </main>

  <StatusBar
    {dirty}
    wordCount={stats.words}
    charCount={stats.chars}
    line={cursor.line}
    col={cursor.col}
    {notice}
  />

  {#if unsavedPrompt}
    <UnsavedDialog name={unsavedPrompt.name} onChoice={unsavedPrompt.resolve} />
  {/if}

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
    position: relative; /* dialogs are positioned against the app, not the screen */
    overflow: hidden;
  }

  :global(html[data-rounded]) .app {
    border-radius: var(--window-radius);
    border: 1px solid var(--border);
  }

  .toolbar {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    height: var(--toolbar-height);
    padding: 0 4px 0 10px;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
    flex: 0 0 auto;
    cursor: default;
    user-select: none;
    font-size: var(--text-min);
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    flex: 1 1 0;
    min-width: 0;
  }

  .toolbar-left {
    gap: 10px;
  }

  .toolbar-right {
    justify-content: flex-end;
  }

  .logo {
    font-size: 18px;
    line-height: 1;
    font-family: serif;
    font-weight: 700;
    color: var(--accent);
    /* Optical correction for the cedilla descender hook */
    transform: translateY(-2px);
  }

  .doc-name {
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty-dot {
    color: var(--warning);
    margin-left: 6px;
    font-size: 9px;
  }

  /* Text tabs; the active one is underlined in the accent color. */
  .tabs {
    display: flex;
    align-items: stretch;
    gap: 2px;
  }

  .tabs button {
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    padding: 0 12px;
    font-size: var(--text-min);
    color: var(--fg-dim);
  }

  .tabs button:hover {
    color: var(--fg);
  }

  .tabs button.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  .tab-hint {
    align-self: center;
    margin-left: 8px;
    color: var(--fg-dim);
    opacity: 0.6;
    font-size: var(--text-min);
  }

  .settings-btn {
    background: transparent;
    border: none;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 0 8px;
    height: 100%;
    font-size: var(--text-min);
    color: var(--fg-dim);
  }

  .settings-btn:hover {
    color: var(--fg);
  }

  .key-badge {
    color: var(--fg-dim);
    opacity: 0.7;
  }

  /* Flat window buttons, only where there is no title bar to provide them. */
  .win-controls {
    display: flex;
    align-items: stretch;
    height: 100%;
    margin-left: 4px;
  }

  .wc {
    background: transparent;
    border: none;
    border-radius: 0;
    width: 34px;
    padding: 0;
    color: var(--fg-dim);
    font-size: var(--text-min);
  }

  .wc:hover {
    color: var(--fg);
    background: var(--bg-surface);
  }

  .wc.close:hover {
    color: var(--bg);
    background: var(--danger);
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
