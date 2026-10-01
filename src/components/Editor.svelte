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

  function findMathMatch(
    docText: string,
    rawTex: string,
    mathMode: "inline" | "display",
    contextBefore = "",
    contextAfter = "",
    approxRatio = 0,
    subtext?: string,
  ): { from: number; to: number } | null {
    if (!rawTex) return null;

    const cleanTex = rawTex.trim();
    const targetOffset = Math.floor(docText.length * approxRatio);
    const candidates: Array<{ idx: number; score: number }> = [];

    let pos = docText.indexOf(cleanTex);
    while (pos !== -1) {
      let score = 0;
      const dist = Math.abs(pos - targetOffset);
      score -= (dist / (docText.length || 1)) * 50;

      let b = pos;
      while (
        b > 0 &&
        (docText[b - 1] === " " || docText[b - 1] === "\t" || docText[b - 1] === "\n")
      ) {
        b--;
      }
      const isEnclosedDisplay = b >= 2 && docText.slice(b - 2, b) === "$$";
      const isEnclosedInline = b >= 1 && docText[b - 1] === "$";

      if (mathMode === "display" && isEnclosedDisplay) score += 100;
      else if (mathMode === "inline" && isEnclosedInline) score += 100;
      else if (isEnclosedInline || isEnclosedDisplay) score += 50;

      if (contextBefore) {
        const windowBefore = docText.slice(Math.max(0, pos - 80), pos);
        const cleanBefore = contextBefore.trim().slice(-25);
        if (cleanBefore && windowBefore.includes(cleanBefore)) {
          score += 200;
        }
      }

      if (contextAfter) {
        const windowAfter = docText.slice(
          pos + cleanTex.length,
          Math.min(docText.length, pos + cleanTex.length + 80),
        );
        const cleanAfter = contextAfter.trim().slice(0, 25);
        if (cleanAfter && windowAfter.includes(cleanAfter)) {
          score += 200;
        }
      }

      candidates.push({ idx: pos, score });
      pos = docText.indexOf(cleanTex, pos + 1);
    }

    if (candidates.length === 0) {
      const partial = cleanTex.slice(0, Math.min(30, cleanTex.length)).trim();
      if (partial.length >= 5) {
        let pPos = docText.indexOf(partial);
        while (pPos !== -1) {
          const dist = Math.abs(pPos - targetOffset);
          candidates.push({ idx: pPos, score: -dist });
          pPos = docText.indexOf(partial, pPos + 1);
        }
      }
    }

    if (candidates.length === 0) return null;

    candidates.sort((a, b) => b.score - a.score);
    const best = candidates[0].idx;

    let from = best;
    let to = best + cleanTex.length;

    let b = best;
    while (
      b > 0 &&
      (docText[b - 1] === " " || docText[b - 1] === "\t" || docText[b - 1] === "\n")
    ) {
      b--;
    }
    if (b >= 2 && docText.slice(b - 2, b) === "$$") {
      from = b - 2;
    } else if (b >= 1 && docText[b - 1] === "$") {
      from = b - 1;
    }

    let e = to;
    while (
      e < docText.length &&
      (docText[e] === " " || docText[e] === "\t" || docText[e] === "\n")
    ) {
      e++;
    }
    if (e + 2 <= docText.length && docText.slice(e, e + 2) === "$$") {
      to = e + 2;
    } else if (e < docText.length && docText[e] === "$") {
      to = e + 1;
    }

    if (subtext && cleanTex.includes(subtext)) {
      const subIdx = cleanTex.indexOf(subtext);
      if (subIdx !== -1) {
        return { from: best + subIdx, to: best + subIdx + subtext.length };
      }
    }

    return { from, to };
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
      let bestEnd = -1;
      while (wPos !== -1) {
        const dist = Math.abs(wPos - targetOffset);
        const windowEnd = Math.min(docText.length, wPos + text.length + 80);
        const slice = docText.slice(wPos, windowEnd);
        const lastIdx = slice.lastIndexOf(lastWord);
        if (lastIdx !== -1 && dist < bestDist) {
          bestDist = dist;
          bestIdx = wPos;
          bestEnd = wPos + lastIdx + lastWord.length;
        }
        wPos = docText.indexOf(firstWord, wPos + 1);
      }
      if (bestIdx !== -1 && bestEnd !== -1) {
        return { from: bestIdx, to: bestEnd };
      }
    }

    return null;
  }

  export function scrollToText(detail: {
    text: string;
    contextBefore?: string;
    contextAfter?: string;
    approxRatio?: number;
    isMath?: boolean;
    mathMode?: "inline" | "display";
    mathSubtext?: string;
  }) {
    if (!view) return;
    const docStr = view.state.doc.toString();
    let match: { from: number; to: number } | null = null;

    if (detail.isMath) {
      match = findMathMatch(
        docStr,
        detail.text,
        detail.mathMode || "inline",
        detail.contextBefore,
        detail.contextAfter,
        detail.approxRatio ?? 0,
        detail.mathSubtext,
      );
    }

    if (!match) {
      match = findBestMatch(
        docStr,
        detail.text,
        detail.contextBefore,
        detail.contextAfter,
        detail.approxRatio ?? 0,
      );
    }

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
