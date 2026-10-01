<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { createEditor } from "../lib/editor";
  import {
    wrapCompartment,
    tabSizeCompartment,
    typewriterCompartment,
    getWrapExtension,
    getTabSizeExtension,
  } from "../lib/codemirror/options";
  import { createTypewriterExtension } from "../lib/codemirror/typewriter";
  import { EditorView } from "@codemirror/view";

  const dispatch = createEventDispatcher<{ change: string }>();

  export let content = "";
  export let fontSize = 15;
  export let tabSize = 2;
  export let wrap = true;
  export let typewriter = true;

  let view: EditorView | null = null;
  let host: HTMLDivElement;
  let isInternalUpdate = false;

  onMount(() => {
    const state = createEditor({ content, fontSize, tabSize, wrap, typewriter });
    view = new EditorView({
      state,
      parent: host,
      dispatch(tr) {
        view?.update([tr]);
        if (tr.docChanged) {
          isInternalUpdate = true;
          content = tr.state.doc.toString();
          dispatch("change", content);
          isInternalUpdate = false;
        }
      },
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
  }

  .editor-host :global(.cm-editor) {
    height: 100%;
    width: 100%;
  }

  .editor-host :global(.cm-scroller) {
    font-family: var(--font-mono);
    font-size: var(--editor-font-size, 15px);
    line-height: 1.6;
    padding: 28px clamp(16px, 5vw, 72px);
    max-width: var(--editor-max-width, 760px);
    margin: 0 auto;
    overflow-y: auto;
  }

  /* Inline math preview styling */
  .editor-host :global(.cm-math-inline) {
    background: var(--bg-elev);
    border-radius: 4px;
    padding: 1px 5px;
    display: inline-block;
    transition: background 0.15s ease;
  }

  .editor-host :global(.cm-math-inline:hover) {
    background: var(--bg-surface);
  }

  /* Block math preview styling */
  .editor-host :global(.cm-math-block) {
    display: block;
    margin: 12px 0;
    padding: 10px 16px;
    background: var(--bg-elev);
    border-radius: var(--radius);
    text-align: center;
    overflow-x: auto;
  }

  .editor-host :global(.cm-math-error) {
    color: var(--danger);
    font-family: var(--font-mono);
    font-size: 0.9em;
  }
</style>
