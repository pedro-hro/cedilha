<script lang="ts">
  export let dirty = false;
  export let wordCount = 0;
  export let charCount = 0;
  export let line = 1;
  export let col = 1;
  export let notice = "";
</script>

<footer class="status-bar" role="status" aria-live="polite">
  <span class="seg pos">ln {line}, col {col}</span>
  <span class="seg" class:dirty class:clean={!dirty}>{dirty ? "● unsaved" : "✓ saved"}</span>
  {#if notice}<span class="seg error">{notice}</span>{/if}

  <span class="spacer"></span>

  <span class="seg">{wordCount.toLocaleString()} words</span>
  <span class="seg">{charCount.toLocaleString()} chars</span>
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    height: var(--statusbar-height);
    padding: 0 4px;
    background: var(--bg-elev);
    border-top: 1px solid var(--border);
    font-size: var(--text-min);
    color: var(--fg-dim);
    user-select: none;
    overflow: hidden;
    flex: 0 0 auto;
  }

  .seg {
    display: inline-flex;
    align-items: center;
    padding: 0 10px;
    white-space: nowrap;
  }

  .seg + .seg {
    border-left: 1px solid var(--border);
  }

  .spacer {
    flex: 1 1 auto;
    border: none;
  }

  .spacer + .seg {
    border-left: none;
  }

  .pos {
    color: var(--fg);
  }

  .dirty {
    color: var(--warning);
  }

  .clean {
    color: var(--success);
  }

  .error {
    color: var(--danger);
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
