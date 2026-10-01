<script lang="ts">
  import katex from "katex";

  export let latex = "";
  export let display = false;

  let html = "";
  let error = "";

  $: {
    const trimmed = latex.trim();
    if (!trimmed) {
      html = "";
      error = "";
    } else {
      try {
        html = katex.renderToString(trimmed, {
          throwOnError: false,
          displayMode: display,
          output: "html",
        });
        error = "";
      } catch (e) {
        html = "";
        error = (e as Error).message;
      }
    }
  }
</script>

<div class="math-preview" class:error>
  {#if error}
    <span class="math-error">⚠ {error}</span>
  {:else if html}
    <div class="katex-render">{@html html}</div>
  {/if}
</div>

<style>
  .math-preview {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 1.2em;
    padding: 2px 6px;
    border-radius: var(--radius, 4px);
    background: var(--bg-elev);
  }

  .math-preview.error {
    border: 1px dashed var(--danger);
  }

  .math-error {
    color: var(--danger);
    font-size: 0.85em;
    white-space: normal;
  }

  .katex-render {
    overflow-x: auto;
    max-width: 100%;
  }
</style>
