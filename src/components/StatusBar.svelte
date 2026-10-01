<script lang="ts">
  export let path: string | null = null;
  export let dirty = false;
  export let wordCount = 0;
  export let charCount = 0;
  export let lineCount = 0;

  function basename(p: string): string {
    return p.split(/[/\\]/).pop() || p;
  }

  // ~200 words per minute reading estimate
  const WPM = 200;
  $: minutes = Math.max(1, Math.round(wordCount / WPM));
  $: readingTime = wordCount === 0 ? "—" : `${minutes} min read`;

  $: statusSegments = [
    { label: path ? basename(path) : "new document", cls: "path" },
    { label: dirty ? "● unsaved" : "✓ saved", cls: dirty ? "dirty" : "clean" },
    { label: `${wordCount.toLocaleString()} words` },
    { label: `${charCount.toLocaleString()} chars` },
    { label: `${lineCount.toLocaleString()} lines` },
    { label: readingTime, cls: "muted" },
  ];
</script>

<footer class="status-bar" role="status" aria-live="polite">
  {#each statusSegments as seg, i (seg.label)}
    <span class="seg {seg.cls}">
      {#if i > 0}<span class="dot">·</span>{/if}{seg.label}
    </span>
  {/each}
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    height: var(--statusbar-height, 26px);
    padding: 0 12px;
    background: var(--bg-elev);
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--fg-dim);
    user-select: none;
    overflow: hidden;
    flex: 0 0 auto;
  }

  .seg {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 0 6px;
    border-radius: 3px;
  }

  .dot {
    opacity: 0.5;
  }

  .path {
    color: var(--fg);
    font-weight: 600;
    max-width: 40%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty {
    color: var(--warning);
  }

  .clean {
    color: var(--success);
  }

  .muted {
    margin-left: auto;
    opacity: 0.8;
  }
</style>
