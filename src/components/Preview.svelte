<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import katex from "katex";
  import * as api from "../lib/tauri";

  export let content = "";
  export let centered = false;

  let renderedHtml = "";
  let container: HTMLElement;
  let scrollContainer: HTMLDivElement;
  let mathObserver: IntersectionObserver | null = null;
  let idleTimeout: ReturnType<typeof setTimeout> | null = null;
  let debounceTimeout: ReturnType<typeof setTimeout> | null = null;
  let currentRequestId = 0;

  function renderKatexElement(el: HTMLElement) {
    if (el.dataset.katex) return;
    el.dataset.katex = "1";
    const raw = el.textContent || "";
    const isDisplay = el.classList.contains("math-display");
    try {
      katex.render(raw.trim(), el, {
        displayMode: isDisplay,
        throwOnError: false,
      });
    } catch {
      // Keep fallback unrendered text on syntax error
    }
  }

  function setupMathRendering() {
    if (!container) return;

    if (mathObserver) {
      mathObserver.disconnect();
      mathObserver = null;
    }
    if (idleTimeout) {
      clearTimeout(idleTimeout);
      idleTimeout = null;
    }

    const mathEls = container.querySelectorAll<HTMLElement>(
      ".math-inline:not([data-katex]), .math-display:not([data-katex])",
    );

    if (mathEls.length === 0) return;

    // For documents with few equations (<= 40), render all immediately in one go
    if (mathEls.length <= 40) {
      for (let i = 0; i < mathEls.length; i++) {
        renderKatexElement(mathEls[i]);
      }
      return;
    }

    // For large documents (e.g. books with thousands of equations):
    // 1. Observe visible equations in viewport with 400px margin for instant display
    if (typeof IntersectionObserver !== "undefined") {
      mathObserver = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            if (entry.isIntersecting) {
              const target = entry.target as HTMLElement;
              mathObserver?.unobserve(target);
              renderKatexElement(target);
            }
          }
        },
        {
          root: scrollContainer || null,
          rootMargin: "400px 0px 400px 0px",
        },
      );

      for (let i = 0; i < mathEls.length; i++) {
        mathObserver.observe(mathEls[i]);
      }
    }

    // 2. Background progressive rendering for any off-screen equations
    let idleIndex = 0;
    function processIdleBatch() {
      if (!container) return;
      const pending = container.querySelectorAll<HTMLElement>(
        ".math-inline:not([data-katex]), .math-display:not([data-katex])",
      );
      if (pending.length === 0) return;

      const startTime = performance.now();
      while (idleIndex < pending.length && performance.now() - startTime < 10) {
        renderKatexElement(pending[idleIndex++]);
      }

      if (idleIndex < pending.length) {
        idleTimeout = setTimeout(processIdleBatch, 20);
      }
    }

    idleTimeout = setTimeout(processIdleBatch, 100);
  }

  async function updatePreview(raw: string, immediate = false) {
    if (debounceTimeout) {
      clearTimeout(debounceTimeout);
      debounceTimeout = null;
    }

    if (!raw.trim()) {
      renderedHtml = '<div class="preview-empty">No content to preview</div>';
      return;
    }

    const reqId = ++currentRequestId;

    const doRender = async () => {
      try {
        const html = await api.renderMarkdown(raw);
        if (reqId === currentRequestId) {
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

    if (immediate) {
      await doRender();
    } else {
      debounceTimeout = setTimeout(doRender, 150);
    }
  }

  $: {
    updatePreview(content, renderedHtml === "");
  }

  onDestroy(() => {
    if (debounceTimeout) clearTimeout(debounceTimeout);
    if (idleTimeout) clearTimeout(idleTimeout);
    if (mathObserver) mathObserver.disconnect();
  });
</script>

<div bind:this={scrollContainer} class="preview-container" class:centered>
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
    line-height: 1.7;
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
