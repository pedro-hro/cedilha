// Typewriter / center-line scrolling extension for CodeMirror 6.
// Keeps the active cursor line vertically centered on the viewport while typing.

import type { Extension } from "@codemirror/state";
import { EditorView, type ViewUpdate } from "@codemirror/view";

/**
 * Creates an extension that centers the active line per keystroke when enabled.
 */
export function createTypewriterExtension(enabled: boolean): Extension {
  if (!enabled) {
    return [];
  }

  let rafId: number | null = null;

  return EditorView.updateListener.of((update: ViewUpdate) => {
    // Only center when typing or single cursor move, not while dragging selection
    const isSingleCursor = update.state.selection.main.empty;
    if ((update.docChanged || (update.selectionSet && isSingleCursor)) && update.view.hasFocus) {
      const head = update.state.selection.main.head;
      const view = update.view;
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }
      rafId = requestAnimationFrame(() => {
        rafId = null;
        try {
          view.dispatch({
            effects: EditorView.scrollIntoView(head, { y: "center" }),
          });
        } catch {
          // View may have been destroyed during unmount
        }
      });
    }
  });
}
