// Shared CodeMirror compartments and options for reactive editor configuration.
import { Compartment, type Extension } from "@codemirror/state";
import { lineNumbers, highlightActiveLineGutter, keymap, type KeyBinding } from "@codemirror/view";
import {
  findNext,
  findPrevious,
  closeSearchPanel,
  selectSelectionMatches,
  selectNextOccurrence,
  gotoLine,
} from "@codemirror/search";
import { toCodeMirror } from "../keybindings";
import { EditorState } from "@codemirror/state";

export interface EditorOptions {
  fontSize: number;
  tabSize: number;
  typewriter: boolean;
  lineNumbers?: boolean;
  findNextKeys?: string[];
  findPrevKeys?: string[];
}

export const tabSizeCompartment = new Compartment();
export const typewriterCompartment = new Compartment();
export const lineNumbersCompartment = new Compartment();
export const searchKeysCompartment = new Compartment();

export function getTabSizeExtension(tabSize: number): Extension {
  return EditorState.tabSize.of(tabSize);
}

export function getLineNumbersExtension(enabled: boolean): Extension {
  return enabled ? [lineNumbers(), highlightActiveLineGutter()] : [];
}

// Find next/previous follow the user's keybindings; the rest of the search keymap is fixed.
// (Opening the panel itself is handled app-wide, see App.svelte.)
export function getSearchKeysExtension(next: string[] = [], prev: string[] = []): Extension {
  const scope = "editor search-panel";
  const bindings: KeyBinding[] = [];
  for (const k of next) {
    const key = toCodeMirror(k);
    if (key) bindings.push({ key, run: findNext, scope, preventDefault: true });
  }
  for (const k of prev) {
    const key = toCodeMirror(k);
    if (key) bindings.push({ key, run: findPrevious, scope, preventDefault: true });
  }
  bindings.push(
    { key: "Escape", run: closeSearchPanel, scope },
    { key: "Mod-Shift-l", run: selectSelectionMatches },
    { key: "Mod-Alt-g", run: gotoLine },
    { key: "Mod-d", run: selectNextOccurrence, preventDefault: true },
  );
  return keymap.of(bindings);
}
