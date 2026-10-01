// Shared CodeMirror compartments and options for reactive editor configuration.
import { Compartment, type Extension } from "@codemirror/state";
import { EditorView, lineNumbers, highlightActiveLineGutter } from "@codemirror/view";
import { EditorState } from "@codemirror/state";

export interface EditorOptions {
  fontSize: number;
  tabSize: number;
  wrap: boolean;
  typewriter: boolean;
  lineNumbers?: boolean;
}

export const tabSizeCompartment = new Compartment();
export const wrapCompartment = new Compartment();
export const typewriterCompartment = new Compartment();
export const lineNumbersCompartment = new Compartment();

export function getWrapExtension(wrap: boolean): Extension {
  return wrap ? EditorView.lineWrapping : [];
}

export function getTabSizeExtension(tabSize: number): Extension {
  return EditorState.tabSize.of(tabSize);
}

export function getLineNumbersExtension(enabled: boolean): Extension {
  return enabled ? [lineNumbers(), highlightActiveLineGutter()] : [];
}
