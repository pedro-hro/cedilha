<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { createEditor } from "../lib/editor";
  import {
    tabSizeCompartment,
    typewriterCompartment,
    lineNumbersCompartment,
    searchKeysCompartment,
    getSearchKeysExtension,
    getTabSizeExtension,
    getLineNumbersExtension,
  } from "../lib/codemirror/options";
  import { createTypewriterExtension } from "../lib/codemirror/typewriter";
  import { EditorView } from "@codemirror/view";
  import { openSearchPanel } from "@codemirror/search";

  const dispatch = createEventDispatcher<{
    change: string;
    scroll: void;
  }>();

  export let content = "";
  export let fontSize = 15;
  export let tabSize = 2;
  export let typewriter = true;
  export let lineNumbers = false;
  export let findNextKeys: string[] = [];
  export let findPrevKeys: string[] = [];

  let view: EditorView | null = null;
  let host: HTMLDivElement;

  function buildListener() {
    return EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        content = update.state.doc.toString();
        dispatch("change", content);
      }
    });
  }

  onMount(() => {
    const updateListener = buildListener();
    const state = createEditor({
      content,
      fontSize,
      tabSize,
      typewriter,
      lineNumbers,
      findNextKeys,
      findPrevKeys,
      updateListener,
    });

    view = new EditorView({
      state,
      parent: host,
    });

    const onScroll = () => dispatch("scroll");

    view.scrollDOM.addEventListener("scroll", onScroll, { passive: true });

    return () => {
      view?.scrollDOM.removeEventListener("scroll", onScroll);
      view?.destroy();
      view = null;
    };
  });

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

  $: if (view) {
    view.dispatch({
      effects: searchKeysCompartment.reconfigure(getSearchKeysExtension(findNextKeys, findPrevKeys)),
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

  export function openSearch() {
    if (!view) return;
    view.focus();
    openSearchPanel(view);
  }

  // Source offset (in the document) of the line at the top of the viewport.
  export function getScrollAnchor(): number {
    if (!view) return 0;
    const scroller = view.scrollDOM;
    const y = scroller.getBoundingClientRect().top - view.documentTop;
    const block = view.lineBlockAtHeight(y);
    const frac = block.height > 0 ? Math.min(1, Math.max(0, (y - block.top) / block.height)) : 0;
    return block.from + frac * block.length;
  }

  // Scroll so that the given source offset sits at the top of the viewport.
  export function scrollToAnchor(pos: number) {
    if (!view) return;
    const scroller = view.scrollDOM;
    if (pos <= 0) {
      scroller.scrollTop = 0;
      return;
    }
    const p = Math.min(pos, view.state.doc.length);
    const block = view.lineBlockAt(p);
    const frac = block.length > 0 ? (p - block.from) / block.length : 0;
    const y = block.top + frac * block.height;
    const offset = view.documentTop - scroller.getBoundingClientRect().top + scroller.scrollTop;
    scroller.scrollTop = y + offset;
  }

  export function focusEditor() {
    view?.focus();
  }

  export function setContent(c: string) {
    content = c;
    if (view) {
      const state = createEditor({
        content: c,
        fontSize,
        tabSize,
        typewriter,
        lineNumbers,
        findNextKeys,
        findPrevKeys,
        updateListener: buildListener(),
      });
      view.setState(state);
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
    justify-content: flex-start;
  }

  .editor-host :global(.cm-editor) {
    height: 100%;
    width: 100%;
    max-width: 100%;
    display: flex;
    flex-direction: column;
  }

  .editor-host :global(.cm-scroller) {
    flex: 1 1 auto;
    height: 100%;
    width: 100%;
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: var(--editor-font-size, 15px);
    line-height: 1.6;
    padding: 12px 0 240px 0;
    display: flex;
    align-items: flex-start;
    box-sizing: border-box;
  }

  /* Full-width content so line highlights run edge to edge (Zed/Sublime style). The
     reading column is enforced by the lines' right padding instead, see lib/editor.ts. */
  .editor-host :global(.cm-content) {
    flex: 1 1 auto;
    width: 100%;
    max-width: 100%;
    padding: 0;
    margin: 0;
    box-sizing: border-box;
    outline: none;
    caret-color: var(--accent);
  }
</style>
