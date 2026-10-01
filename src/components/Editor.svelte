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

  const dispatch = createEventDispatcher<{
    change: string;
    scroll: { ratio: number };
  }>();

  export let content = "";
  export let fontSize = 15;
  export let tabSize = 2;
  export let typewriter = true;
  export let lineNumbers = false;

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
      updateListener,
    });

    view = new EditorView({
      state,
      parent: host,
    });

    const onScroll = () => {
      if (!view) return;
      const scroller = view.scrollDOM;
      const max = scroller.scrollHeight - scroller.clientHeight;
      const ratio = max > 0 ? scroller.scrollTop / max : 0;
      dispatch("scroll", { ratio });
    };

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
    if (view) {
      const state = createEditor({
        content: c,
        fontSize,
        tabSize,
        typewriter,
        lineNumbers,
        updateListener: buildListener(),
      });
      view.setState(state);
    }
  }

  function findBestMatch(
    docText: string,
    text: string,
    contextBefore = "",
    contextAfter = "",
    approxRatio = 0,
  ): { from: number; to: number } | null {
    if (!text) return null;

    if (contextBefore && contextAfter) {
      const query = contextBefore + text + contextAfter;
      const idx = docText.indexOf(query);
      if (idx !== -1) {
        return {
          from: idx + contextBefore.length,
          to: idx + contextBefore.length + text.length,
        };
      }
    }

    if (contextBefore) {
      const query = contextBefore.slice(-15) + text;
      const idx = docText.indexOf(query);
      if (idx !== -1) {
        return {
          from: idx + contextBefore.slice(-15).length,
          to: idx + contextBefore.slice(-15).length + text.length,
        };
      }
    }

    const targetOffset = Math.floor(docText.length * approxRatio);
    let bestIdx = -1;
    let bestDist = Infinity;

    let pos = docText.indexOf(text);
    while (pos !== -1) {
      const dist = Math.abs(pos - targetOffset);
      if (dist < bestDist) {
        bestDist = dist;
        bestIdx = pos;
      }
      pos = docText.indexOf(text, pos + 1);
    }

    if (bestIdx !== -1) {
      return { from: bestIdx, to: bestIdx + text.length };
    }

    const words = text.split(/\s+/).filter(Boolean);
    if (words.length >= 2) {
      const firstWord = words[0];
      const lastWord = words[words.length - 1];
      let wPos = docText.indexOf(firstWord);
      while (wPos !== -1) {
        const dist = Math.abs(wPos - targetOffset);
        const windowEnd = Math.min(docText.length, wPos + text.length + 50);
        const slice = docText.slice(wPos, windowEnd);
        if (slice.includes(lastWord) && dist < bestDist) {
          bestDist = dist;
          bestIdx = wPos;
        }
        wPos = docText.indexOf(firstWord, wPos + 1);
      }
      if (bestIdx !== -1) {
        return { from: bestIdx, to: bestIdx + text.length };
      }
    }

    return null;
  }

  export function scrollToText(detail: {
    text: string;
    contextBefore?: string;
    contextAfter?: string;
    approxRatio?: number;
  }) {
    if (!view) return;
    const docStr = view.state.doc.toString();
    const match = findBestMatch(
      docStr,
      detail.text,
      detail.contextBefore,
      detail.contextAfter,
      detail.approxRatio ?? 0,
    );
    if (match) {
      view.dispatch({
        selection: { anchor: match.from, head: match.to },
        effects: [EditorView.scrollIntoView(match.from, { y: "center" })],
      });
      view.focus();
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
    padding: 24px 32px 240px 24px;
    display: flex;
    align-items: flex-start;
    box-sizing: border-box;
  }

  .editor-host :global(.cm-content) {
    flex: 1 1 auto;
    width: 100%;
    max-width: var(--editor-max-width, 1040px);
    padding: 0;
    margin: 0;
    box-sizing: border-box;
    outline: none;
    caret-color: var(--accent);
  }
</style>
