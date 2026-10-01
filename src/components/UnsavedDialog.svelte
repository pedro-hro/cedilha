<script lang="ts">
  // In-app "unsaved changes" prompt. Needs no zenity/kdialog/portal, so it works the same on
  // every distro, and it is fully keyboard driven (S / D / C, arrows + Enter, Esc cancels).
  import { onMount, onDestroy } from "svelte";
  import type { UnsavedChoice } from "../lib/tauri";

  export let name = "";
  export let onChoice: (choice: UnsavedChoice) => void;

  const buttons: { choice: UnsavedChoice; label: string; key: string }[] = [
    { choice: "save", label: "Save", key: "S" },
    { choice: "discard", label: "Discard", key: "D" },
    { choice: "cancel", label: "Cancel", key: "C" },
  ];

  let sel = 0;
  let listening = false;

  function onKey(e: KeyboardEvent) {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    let handled = true;
    const k = e.key.toLowerCase();
    if (k === "s") onChoice("save");
    else if (k === "d") onChoice("discard");
    else if (k === "c" || k === "escape" || k === "q") onChoice("cancel");
    else if (k === "enter" || k === " ") onChoice(buttons[sel].choice);
    else if (k === "arrowright" || k === "l" || (k === "tab" && !e.shiftKey)) sel = (sel + 1) % buttons.length;
    else if (k === "arrowleft" || k === "h" || (k === "tab" && e.shiftKey)) sel = (sel + buttons.length - 1) % buttons.length;
    else handled = false;
    if (handled) {
      e.preventDefault();
      e.stopImmediatePropagation();
    }
  }

  // Capture phase, so neither the editor nor the app shortcuts see these keys.
  onMount(() => {
    (document.activeElement as HTMLElement | null)?.blur();
    window.addEventListener("keydown", onKey, true);
    listening = true;
  });
  onDestroy(() => {
    if (listening) window.removeEventListener("keydown", onKey, true);
  });
</script>

<div class="scrim">
  <div class="panel" role="alertdialog" aria-label="Unsaved changes">
    <div class="title">unsaved changes</div>
    <p><span class="name">{name}</span> has changes that are not saved.</p>
    <div class="buttons">
      {#each buttons as b, i}
        <button class:sel={i === sel} onclick={() => onChoice(b.choice)} onmouseenter={() => (sel = i)}>
          <kbd>{b.key}</kbd> {b.label}
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 18vh;
    z-index: 200;
  }

  .panel {
    width: min(440px, 92vw);
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: var(--text-min);
    padding: 12px 16px 14px;
  }

  .title {
    color: var(--warning);
    font-weight: 700;
  }

  p {
    margin: 10px 0 14px;
    color: var(--fg);
  }

  .name {
    color: var(--accent);
  }

  .buttons {
    display: flex;
    gap: 6px;
  }

  button {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--fg-dim);
    padding: 4px 12px;
    font: inherit;
    cursor: pointer;
  }

  button.sel {
    color: var(--accent);
    border-color: var(--accent);
  }

  kbd {
    font-family: var(--font-mono);
    color: inherit;
    opacity: 0.7;
    margin-right: 4px;
  }
</style>
