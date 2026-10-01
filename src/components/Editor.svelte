<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { createEditor } from "../lib/editor";
  import {
    tabSizeCompartment,
    typewriterCompartment,
    lineNumbersCompartment,
    getTabSizeExtension,
    getLineNumbersExtension,
  } from "../lib/codemirror/options";
  import { createTypewriterExtension } from "../lib/codemirror/typewriter";
  import { EditorView } from "@codemirror/view";

  const dispatch = createEventDispatcher<{ change: string }>();

  export let content = "";
  export let fontSize = 15;
  export let tabSize = 2;
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
    justify-content: center;
  }

  .editor-host :global(.cm-editor) {
    height: 100%;
    width: 100%;
    max-width: var(--editor-max-width, 780px);
    display: flex;
    flex-direction: column;
  }

  .editor-host :global(.cm-scroller) {
    flex: 1 1 auto;
    height: 100%;
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: var(--editor-font-size, 15px);
    line-height: 1.65;
    padding: 36px 16px 240px 16px;
    display: flex;
    align-items: flex-start;
  }

  .editor-host :global(.cm-content) {
    flex: 1 1 auto;
    width: 100%;
    padding: 0;
    margin: 0;
    box-sizing: border-box;
    outline: none;
    caret-color: var(--accent);
  }
</style>
