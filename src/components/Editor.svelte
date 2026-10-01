<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { createEditor } from "../lib/editor";
  import {
    wrapCompartment,
    tabSizeCompartment,
    typewriterCompartment,
    lineNumbersCompartment,
    getWrapExtension,
    getTabSizeExtension,
    getLineNumbersExtension,
  } from "../lib/codemirror/options";
  import { createTypewriterExtension } from "../lib/codemirror/typewriter";
  import { EditorView } from "@codemirror/view";

  const dispatch = createEventDispatcher<{ change: string }>();

  export let content = "";
  export let fontSize = 15;
  export let tabSize = 2;
  export let wrap = true;
  export let typewriter = true;
  export let lineNumbers = false;

  let view: EditorView | null = null;
  let host: HTMLDivElement;
  let isInternalUpdate = false;

  onMount(() => {
    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        isInternalUpdate = true;
        content = update.state.doc.toString();
        dispatch("change", content);
        isInternalUpdate = false;
      }
    });

    const state = createEditor({
      content,
      fontSize,
      tabSize,
      wrap,
      typewriter,
      lineNumbers,
      updateListener,
    });

    view = new EditorView({
      state,
      parent: host,
    });

    return () => {
      view?.destroy();
      view = null;
    };
  });

  // Sync external content changes into CodeMirror (e.g. Open File, New File)
  $: if (view && !isInternalUpdate && content !== view.state.doc.toString()) {
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: content },
    });
  }

  // Reactive options dispatch via CodeMirror compartments
  $: if (view && wrap !== undefined) {
    view.dispatch({
      effects: wrapCompartment.reconfigure(getWrapExtension(wrap)),
    });
  }

  $: if (view && tabSize !== undefined) {
    view.dispatch({
      effects: tabSizeCompartment.reconfigure(getTabSizeExtension(tabSize)),
    });
  }

  $: if (view && typewriter !== undefined) {
    view.dispatch({
      effects: typewriterCompartment.reconfigure(createTypewriterExtension(typewriter)),
    });
  }

  $: if (view && lineNumbers !== undefined) {
    view.dispatch({
      effects: lineNumbersCompartment.reconfigure(getLineNumbersExtension(lineNumbers)),
    });
  }

  export function getContent(): string {
    return view?.state.doc.toString() ?? content;
  }

  export function focusEditor() {
    view?.focus();
  }

  export function setContent(c: string) {
    content = c;
    if (view && view.state.doc.toString() !== c) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: c },
      });
    }
  }
</script>

<div bind:this={host} class="editor-host" style="--editor-font-size: {fontSize}px;"></div>

<style>
  .editor-host {
    flex: 1 1 auto;
    height: 100%;
    width: 100%;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .editor-host :global(.cm-editor) {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
  }

  .editor-host :global(.cm-scroller) {
    height: 100%;
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: var(--editor-font-size, 15px);
    line-height: 1.7;
    display: flex;
  }

  .editor-host :global(.cm-gutters) {
    background: transparent;
    color: var(--fg-dim);
    border-right: 1px solid var(--border);
    padding: 36px 8px 36px 16px;
    user-select: none;
    flex-shrink: 0;
  }

  .editor-host :global(.cm-content) {
    flex: 1 1 auto;
    max-width: var(--editor-max-width, 780px);
    margin: 0 auto;
    padding: 36px 32px 240px 32px;
    box-sizing: border-box;
    outline: none;
    caret-color: var(--accent);
  }

  .editor-host :global(.cm-line) {
    padding: 0 4px;
  }

  /* Inline math preview styling */
  .editor-host :global(.cm-math-inline) {
    background: var(--bg-elev);
    border-radius: 4px;
    padding: 1px 6px;
    display: inline-block;
    transition: background 0.15s ease;
    border: 1px solid var(--border);
  }

  .editor-host :global(.cm-math-inline:hover) {
    background: var(--bg-surface);
    border-color: var(--accent);
  }

  /* Block math preview styling */
  .editor-host :global(.cm-math-block) {
    display: block;
    margin: 12px auto;
    padding: 12px 18px;
    background: var(--bg-elev);
    border-radius: var(--radius);
    text-align: center;
    overflow-x: auto;
    max-width: 90%;
    border: 1px solid var(--border);
  }

  .editor-host :global(.cm-math-error) {
    color: var(--danger);
    font-family: var(--font-mono);
    font-size: 0.9em;
  }
</style>
