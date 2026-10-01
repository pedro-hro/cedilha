<script lang="ts">
  import { marked } from "marked";
  import katex from "katex";

  export let content = "";
  export let centered = false;

  function renderMarkdown(md: string): string {
    if (!md.trim()) return '<div class="preview-empty">No content to preview</div>';

    // 1. Extract block math $$...$$
    const mathBlocks: string[] = [];
    let text = md.replace(/\$\$([\s\S]+?)\$\$/g, (_, math) => {
      const id = `@@MATHBLOCK_${mathBlocks.length}@@`;
      try {
        mathBlocks.push(
          katex.renderToString(math.trim(), {
            displayMode: true,
            throwOnError: false,
          }),
        );
      } catch (e) {
        mathBlocks.push(`<pre class="math-error">${math}</pre>`);
      }
      return `\n\n${id}\n\n`;
    });

    // 2. Extract inline math $...$
    const mathInlines: string[] = [];
    text = text.replace(/(?<!\\)\$([^\$\n]+?)\$/g, (_, math) => {
      const id = `@@MATHINLINE_${mathInlines.length}@@`;
      try {
        mathInlines.push(
          katex.renderToString(math.trim(), {
            displayMode: false,
            throwOnError: false,
          }),
        );
      } catch (e) {
        mathInlines.push(`<code>${math}</code>`);
      }
      return id;
    });

    // 3. Parse Markdown
    try {
      let html = marked.parse(text) as string;

      // 4. Restore block math
      html = html.replace(/@@MATHBLOCK_(\d+)@@/g, (_, i) => mathBlocks[parseInt(i)] ?? "");

      // 5. Restore inline math
      html = html.replace(/@@MATHINLINE_(\d+)@@/g, (_, i) => mathInlines[parseInt(i)] ?? "");

      return html;
    } catch {
      return `<p>${text}</p>`;
    }
  }

  $: renderedHtml = renderMarkdown(content);
</script>

<div class="preview-container" class:centered>
  <article class="markdown-body">
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

  .markdown-body :global(.katex-display) {
    margin: 1.4em 0;
    padding: 12px 16px;
    background: var(--bg-elev);
    border-radius: var(--radius);
    border: 1px solid var(--border);
    overflow-x: auto;
  }
</style>
