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

  return EditorView.updateListener.of((update: ViewUpdate) => {
    if ((update.docChanged || update.selectionSet) && update.view.hasFocus) {
      const head = update.state.selection.main.head;
      const view = update.view;
      requestAnimationFrame(() => {
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
