<script lang="ts">
  import { onDestroy, tick, createEventDispatcher } from "svelte";
  import katex from "katex";
  import * as api from "../lib/tauri";

  export let content = "";
  export let centered = false;
  export let active = true;

  const dispatch = createEventDispatcher<{
    previewSelect: {
      text: string;
      contextBefore: string;
      contextAfter: string;
      approxRatio: number;
    };
    scroll: { ratio: number };
  }>();

  let renderedHtml = "";
  let container: HTMLElement;
  let scrollContainer: HTMLDivElement;
  let mathObserver: IntersectionObserver | null = null;
  let debounceTimeout: ReturnType<typeof setTimeout> | null = null;
  let currentRequestId = 0;
  let lastRenderedRaw = "";
  let needsRender = false;

  // Cache rendered KaTeX formulas to prevent re-parsing thousands of identical expressions
  const mathCache = new Map<string, string>();

  // Budgeted render queue to guarantee 60fps even during rapid scrolling
  let renderQueue: HTMLElement[] = [];
  let queueRaf: number | null = null;

  function renderKatexElement(el: HTMLElement) {
    if (el.dataset.katex) return;
    el.dataset.katex = "1";
    const raw = (el.textContent || "").trim();
    if (!raw) return;

    const isDisplay = el.classList.contains("math-display");
    const cacheKey = `${isDisplay ? "D:" : "I:"}${raw}`;

    const cached = mathCache.get(cacheKey);
    if (cached !== undefined) {
      el.innerHTML = cached;
      return;
    }

    try {
      const html = katex.renderToString(raw, {
        displayMode: isDisplay,
        throwOnError: false,
        strict: false,
        trust: true,
      });
      mathCache.set(cacheKey, html);
      el.innerHTML = html;
    } catch {
      // Keep fallback text on parse failure
    }
  }

  let isScrolling = false;
  let isSyncing = false;
  let scrollSettleTimer: ReturnType<typeof setTimeout> | null = null;

  function onScroll() {
    isScrolling = true;
    if (!isSyncing && scrollContainer) {
      const max = scrollContainer.scrollHeight - scrollContainer.clientHeight;
      const ratio = max > 0 ? scrollContainer.scrollTop / max : 0;
      dispatch("scroll", { ratio });
    }
    if (scrollSettleTimer) clearTimeout(scrollSettleTimer);
    scrollSettleTimer = setTimeout(() => {
      isScrolling = false;
      if (renderQueue.length > 0 && queueRaf === null) {
        queueRaf = requestAnimationFrame(flushRenderQueue);
      }
    }, 100);
  }

  export function syncScrollRatio(ratio: number) {
    if (!scrollContainer) return;
    const max = scrollContainer.scrollHeight - scrollContainer.clientHeight;
    if (max > 0) {
      isSyncing = true;
      scrollContainer.scrollTop = ratio * max;
      requestAnimationFrame(() => {
        isSyncing = false;
      });
    }
  }

  function flushRenderQueue() {
    queueRaf = null;
    if (isScrolling) {
      // Never mutate DOM during active scroll motion; wait until scroll settles
      return;
    }
    const start = performance.now();
    while (renderQueue.length > 0 && performance.now() - start < 8) {
      const el = renderQueue.shift()!;
      renderKatexElement(el);
    }
    if (renderQueue.length > 0) {
      queueRaf = requestAnimationFrame(flushRenderQueue);
    }
  }

  function setupMathRendering() {
    if (!container) return;

    if (mathObserver) {
      mathObserver.disconnect();
      mathObserver = null;
    }
    renderQueue = [];
    if (queueRaf !== null) {
      cancelAnimationFrame(queueRaf);
      queueRaf = null;
    }

    const mathEls = container.querySelectorAll<HTMLElement>(
      ".math-inline:not([data-katex]), .math-display:not([data-katex])",
    );

    if (mathEls.length === 0) return;

    // For short documents (<= 30 equations), render immediately in one tick
    if (mathEls.length <= 30) {
      for (let i = 0; i < mathEls.length; i++) {
        renderKatexElement(mathEls[i]);
      }
      return;
    }

    // For large documents, strictly render on-demand as equations approach the viewport
    if (typeof IntersectionObserver !== "undefined") {
      mathObserver = new IntersectionObserver(
        (entries) => {
          let hasNew = false;
          for (const entry of entries) {
            if (entry.isIntersecting) {
              const target = entry.target as HTMLElement;
              mathObserver?.unobserve(target);
              renderQueue.push(target);
              hasNew = true;
            }
          }
          if (!isScrolling && hasNew && queueRaf === null) {
            queueRaf = requestAnimationFrame(flushRenderQueue);
          }
        },
        {
          root: scrollContainer || null,
          rootMargin: "1000px 0px 1000px 0px",
        },
      );

      for (let i = 0; i < mathEls.length; i++) {
        mathObserver.observe(mathEls[i]);
      }
    }
  }

  async function updatePreview(raw: string, immediate = false) {
    if (debounceTimeout) {
      clearTimeout(debounceTimeout);
      debounceTimeout = null;
    }

    if (!raw.trim()) {
      renderedHtml = '<div class="preview-empty">No content to preview</div>';
      lastRenderedRaw = "";
      needsRender = false;
      return;
    }

    if (raw === lastRenderedRaw) {
      needsRender = false;
      return;
    }

    if (!active) {
      needsRender = true;
      return;
    }

    const reqId = ++currentRequestId;

    const doRender = async () => {
      try {
        const html = await api.renderMarkdown(raw);
        if (reqId === currentRequestId) {
          lastRenderedRaw = raw;
          needsRender = false;
          renderedHtml = html;
          await tick();
          if (reqId === currentRequestId) {
            setupMathRendering();
          }
        }
      } catch (err) {
        if (reqId === currentRequestId) {
          renderedHtml = `<pre class="preview-empty">Failed to render markdown</pre>`;
        }
      }
    };

    // Adapt debounce based on file size: larger documents (>100KB) debounce by 400ms
    // to avoid layout thrashing during typing, while small documents update in 150ms.
    const delay = immediate ? 0 : raw.length > 100_000 ? 400 : 150;
    if (delay === 0) {
      await doRender();
    } else {
      debounceTimeout = setTimeout(doRender, delay);
    }
  }

  $: {
    // When content changes or when becoming active, update preview
    if (content !== lastRenderedRaw || needsRender) {
      updatePreview(content, renderedHtml === "" || (active && needsRender));
    }
  }

  function handleSelection() {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || !container) return;
    if (!container.contains(sel.anchorNode)) return;

    const text = sel.toString().trim();
    if (!text) return;

    let approxRatio = 0;
    let contextBefore = "";
    let contextAfter = "";

    try {
      const range = sel.getRangeAt(0);
      const rect = range.getBoundingClientRect();
      const containerRect = scrollContainer.getBoundingClientRect();
      const scrollTop = scrollContainer.scrollTop;
      const scrollHeight = scrollContainer.scrollHeight;
      const offset = scrollTop + (rect.top - containerRect.top);
      approxRatio = scrollHeight > 0 ? offset / scrollHeight : 0;

      const block =
        range.startContainer.parentElement?.closest(
          "p, h1, h2, h3, h4, h5, h6, li, blockquote, td, th",
        ) || range.startContainer.parentElement;

      if (block) {
        const preRange = range.cloneRange();
        preRange.setStart(block, 0);
        preRange.setEnd(range.startContainer, range.startOffset);
        contextBefore = preRange.toString().slice(-40);

        const postRange = range.cloneRange();
        postRange.setStart(range.endContainer, range.endOffset);
        postRange.setEnd(block, block.childNodes.length);
        contextAfter = postRange.toString().slice(0, 40);
      }
    } catch {
      // fallback
    }

    dispatch("previewSelect", {
      text,
      contextBefore,
      contextAfter,
      approxRatio,
    });
  }

  onDestroy(() => {
    if (debounceTimeout) clearTimeout(debounceTimeout);
    if (queueRaf !== null) cancelAnimationFrame(queueRaf);
    if (mathObserver) mathObserver.disconnect();
  });
</script>

<div
  bind:this={scrollContainer}
  on:scroll={onScroll}
  on:mouseup={handleSelection}
  on:keyup={handleSelection}
  class="preview-container"
  class:centered
>
  <article bind:this={container} class="markdown-body">
    {@html renderedHtml}
  </article>
</div>

<style>
  .preview-container {
    flex: 1 1 auto;
    height: 100%;
    overflow-y: auto;
    background: var(--bg);
    color: var(--fg);
    user-select: text;
    box-sizing: border-box;
    padding: 24px 32px 240px 32px;
  }

  .markdown-body {
    max-width: var(--editor-max-width, 1040px);
    margin: 0;
    padding: 0;
    font-family: var(--font-mono);
    line-height: 1.6;
    font-size: var(--editor-font-size, 15px);
  }

  .preview-container.centered .markdown-body {
    margin: 0 auto;
  }

  .preview-empty {
    color: var(--fg-dim);
    font-style: italic;
    padding: 40px 0;
    text-align: center;
  }

  .markdown-body :global(h1),
  .markdown-body :global(h2),
  .markdown-body :global(h3),
  .markdown-body :global(h4) {
    color: var(--fg);
    font-weight: 700;
    margin-top: 1.5em;
    margin-bottom: 0.6em;
    line-height: 1.3;
  }

  .markdown-body :global(h1) {
    font-size: 1.85em;
    color: var(--accent);
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.3em;
  }

  .markdown-body :global(h2) {
    font-size: 1.45em;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.25em;
  }

  .markdown-body :global(h3) {
    font-size: 1.2em;
    color: var(--accent-2);
  }

  .markdown-body :global(p) {
    margin-top: 0;
    margin-bottom: 1em;
  }

  .markdown-body :global(a) {
    color: var(--accent);
    text-decoration: none;
    border-bottom: 1px solid transparent;
    transition: border-color 0.15s ease;
  }

  .markdown-body :global(a:hover) {
    border-bottom-color: var(--accent);
  }

  .markdown-body :global(strong) {
    font-weight: 700;
    color: var(--fg);
  }

  .markdown-body :global(code) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    padding: 2px 6px;
    background: var(--bg-elev);
    border-radius: var(--radius);
    border: 1px solid var(--border);
  }

  .markdown-body :global(pre) {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px 20px;
    overflow-x: auto;
    margin: 1.2em 0;
  }

  .markdown-body :global(pre code) {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 0.92em;
  }

  .markdown-body :global(blockquote) {
    margin: 1.2em 0;
    padding: 10px 20px;
    background: var(--bg-elev);
    border-left: 4px solid var(--accent);
    border-radius: 0 var(--radius) var(--radius) 0;
    color: var(--fg-dim);
  }

  .markdown-body :global(blockquote p:last-child) {
    margin-bottom: 0;
  }

  .markdown-body :global(ul),
  .markdown-body :global(ol) {
    padding-left: 24px;
    margin: 1em 0;
  }

  .markdown-body :global(li) {
    margin: 0.35em 0;
  }

  .markdown-body :global(input[type="checkbox"]) {
    margin-right: 6px;
    accent-color: var(--accent);
  }

  .markdown-body :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 2em 0;
  }

  .markdown-body :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1.2em 0;
  }

  .markdown-body :global(th),
  .markdown-body :global(td) {
    padding: 8px 14px;
    border: 1px solid var(--border);
    text-align: left;
  }

  .markdown-body :global(th) {
    background: var(--bg-elev);
    font-weight: 600;
  }

  .markdown-body :global(tr:nth-child(even)) {
    background: var(--bg-elev);
  }

  .markdown-body :global(img) {
    max-width: 100%;
    border-radius: var(--radius);
  }

  .markdown-body :global(.math-inline:not([data-katex])) {
    font-family: var(--font-mono);
    opacity: 0.85;
    padding: 1px 4px;
  }

  .markdown-body :global(.math-display) {
    display: block;
    text-align: center;
    margin: 1.2em 0;
    overflow-x: auto;
  }

  .markdown-body :global(.math-display:not([data-katex])) {
    font-family: var(--font-mono);
    opacity: 0.85;
    padding: 12px 16px;
    background: var(--bg-elev);
    border-radius: var(--radius);
    border: 1px solid var(--border);
    white-space: pre-wrap;
  }

  .markdown-body :global(.katex-display) {
    margin: 1.4em 0;
    padding: 12px 16px;
    background: var(--bg-elev);
    border-radius: var(--radius);
    border: 1px solid var(--border);
    overflow-x: auto;
  }
</style>
