<script lang="ts">
  import { onDestroy, tick, createEventDispatcher } from "svelte";
  import katex from "katex";
  import * as api from "../lib/tauri";

  export let content = "";
  export let centered = false;
  export let active = true;

  const dispatch = createEventDispatcher<{ scroll: void }>();

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
    const raw = (el.dataset.tex || el.textContent || "").trim();
    if (!raw) return;
    el.dataset.tex = raw;
    el.dataset.katex = "1";

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
        trust: false,
      });
      mathCache.set(cacheKey, html);
      el.innerHTML = html;
    } catch {
      // Keep fallback text on parse failure
    }
  }

  let isScrolling = false;
  let scrollSettleTimer: ReturnType<typeof setTimeout> | null = null;

  function onScroll() {
    dispatch("scroll");
    isScrolling = true;
    if (scrollSettleTimer) clearTimeout(scrollSettleTimer);
    scrollSettleTimer = setTimeout(() => {
      isScrolling = false;
      if (renderQueue.length > 0 && queueRaf === null) {
        queueRaf = requestAnimationFrame(flushRenderQueue);
      }
    }, 100);
  }

  // Rendered blocks tagged by the backend with their source range (UTF-16 offsets).
  type Anchor = { el: HTMLElement; start: number; end: number };
  let anchors: Anchor[] | null = null;

  function getAnchors(): Anchor[] {
    if (anchors) return anchors;
    const list: Anchor[] = [];
    container?.querySelectorAll<HTMLElement>("[data-src-start]").forEach((el) => {
      const start = Number(el.dataset.srcStart);
      const end = Number(el.dataset.srcEnd);
      if (!Number.isFinite(start) || !(end > start)) return;
      // The table wrapper is display:contents and has no box of its own.
      const box = el.classList.contains("table-src")
        ? ((el.firstElementChild as HTMLElement | null) ?? el)
        : el;
      list.push({ el: box, start, end });
    });
    anchors = list;
    return list;
  }

  const clamp01 = (n: number) => Math.min(1, Math.max(0, n));

  // Source offset of the block at the top of the viewport.
  export function getScrollAnchor(): number {
    if (!scrollContainer) return 0;
    const list = getAnchors();
    if (list.length === 0) {
      const max = scrollContainer.scrollHeight - scrollContainer.clientHeight;
      return max > 0 ? (scrollContainer.scrollTop / max) * content.length : 0;
    }
    const viewTop = scrollContainer.getBoundingClientRect().top;
    // Last block whose top is at or above the viewport top (document order is top-to-bottom).
    let lo = 0;
    let hi = list.length - 1;
    let found = -1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (list[mid].el.getBoundingClientRect().top <= viewTop + 1) {
        found = mid;
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }
    if (found < 0) return 0;
    const a = list[found];
    const r = a.el.getBoundingClientRect();
    const frac = r.height > 0 ? clamp01((viewTop - r.top) / r.height) : 0;
    return a.start + frac * (a.end - a.start);
  }

  // Scroll so that the given source offset sits at the top of the viewport.
  export function scrollToAnchor(pos: number) {
    if (!scrollContainer) return;
    const list = getAnchors();
    if (list.length === 0) {
      const max = scrollContainer.scrollHeight - scrollContainer.clientHeight;
      if (max > 0 && content.length > 0) scrollContainer.scrollTop = clamp01(pos / content.length) * max;
      return;
    }
    if (pos <= list[0].start) {
      scrollContainer.scrollTop = 0;
      return;
    }
    let lo = 0;
    let hi = list.length - 1;
    let found = 0;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (list[mid].start <= pos) {
        found = mid;
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }
    const a = list[found];
    const frac = clamp01((pos - a.start) / (a.end - a.start));
    const r = a.el.getBoundingClientRect();
    const top = r.top - scrollContainer.getBoundingClientRect().top + scrollContainer.scrollTop;
    scrollContainer.scrollTop = top + frac * r.height;
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

    for (let i = 0; i < mathEls.length; i++) {
      const el = mathEls[i];
      if (!el.dataset.tex) {
        el.dataset.tex = (el.textContent || "").trim();
      }
    }

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
          anchors = null;
          await tick();
          anchors = null;
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

  onDestroy(() => {
    if (debounceTimeout) clearTimeout(debounceTimeout);
    if (queueRaf !== null) cancelAnimationFrame(queueRaf);
    if (mathObserver) mathObserver.disconnect();
  });
</script>

<div
  bind:this={scrollContainer}
  on:scroll={onScroll}
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
    padding: 12px 32px 240px 32px;
  }

  .markdown-body {
    max-width: var(--editor-max-width, 1040px);
    margin: 0;
    padding: 0;
    font-family: var(--font-mono);
    line-height: 1.6;
    font-size: var(--editor-font-size, 15px);
  }

  /* Start flush with the top of the pane so it lines up with the editor in split view. */
  .markdown-body > :global(:first-child) {
    margin-top: 0;
  }

  .preview-container.centered .markdown-body {
    margin: 0 auto;
  }

  .markdown-body :global(.preview-empty) {
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
    font-size: 1.6em;
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
    padding: 1px 5px;
    background: var(--bg-elev);
    border-radius: var(--radius);
  }

  .markdown-body :global(pre) {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px 14px;
    overflow-x: auto;
    margin: 1em 0;
  }

  .markdown-body :global(pre code) {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 0.92em;
  }

  .markdown-body :global(blockquote) {
    margin: 1em 0;
    padding: 0 16px;
    border-left: 2px solid var(--accent);
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

  /* Task-list items carry their own checkbox; drop the bullet next to it. */
  .markdown-body :global(li:has(> input[type="checkbox"])) {
    list-style: none;
    margin-left: -1.4em;
  }

  /* ...but keep a nested list indented under its parent item. */
  .markdown-body :global(li:has(> input[type="checkbox"]) > ul),
  .markdown-body :global(li:has(> input[type="checkbox"]) > ol) {
    margin-left: 1.4em;
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
    padding: 6px 12px;
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
    margin: 1.2em 0;
    overflow-x: auto;
    overflow-y: hidden;
  }
</style>
