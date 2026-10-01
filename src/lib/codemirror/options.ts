// Shared CodeMirror compartments and options for reactive editor configuration.
import { Compartment } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";

export interface EditorOptions {
  fontSize: number;
  tabSize: number;
  wrap: boolean;
  typewriter: boolean;
}

export const tabSizeCompartment = new Compartment();
export const wrapCompartment = new Compartment();
export const typewriterCompartment = new Compartment();

export function getWrapExtension(wrap: boolean) {
  return wrap ? EditorView.lineWrapping : [];
}

export function getTabSizeExtension(tabSize: number) {
  return EditorState.tabSize.of(tabSize);
}
