// Builds a fully-configured CodeMirror 6 editor state for Cedilha.
//
// Composition: Markdown language support + KaTeX math plugin + typewriter
// scrolling + keyboard navigation, with reactive Compartments for runtime settings.

import { EditorState, type Extension } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { standardKeymap, indentWithTab } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { highlightActiveLine } from "@codemirror/view";
import { mathExtension } from "./codemirror/math";
import {
  type EditorOptions,
  wrapCompartment,
  tabSizeCompartment,
  typewriterCompartment,
  lineNumbersCompartment,
  getWrapExtension,
  getTabSizeExtension,
  getLineNumbersExtension,
} from "./codemirror/options";
import { createTypewriterExtension } from "./codemirror/typewriter";

export interface FullEditorOptions extends EditorOptions {
  content: string;
  updateListener?: Extension;
}

// Clean base theme that inherits variables from :root[data-theme] in app.css
const baseTheme = EditorView.theme({
  "&": {
    height: "100%",
    backgroundColor: "transparent",
    color: "var(--fg)",
  },
  ".cm-content": {
    caretColor: "var(--accent)",
    fontFamily: "var(--font-mono)",
  },
  "&.cm-focused .cm-cursor": {
    borderLeftColor: "var(--accent)",
    borderLeftWidth: "2px",
  },
  "&.cm-focused .cm-selectionBackground, ::selection": {
    backgroundColor: "var(--selection) !important",
  },
  ".cm-gutters": {
    backgroundColor: "transparent",
    color: "var(--fg-dim)",
    borderRight: "1px solid var(--border)",
  },
  ".cm-activeLine": {
    backgroundColor: "var(--bg-elev)",
    borderRadius: "var(--radius)",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "transparent",
    color: "var(--accent)",
  },
});

export function createEditor(opts: FullEditorOptions): EditorState {
  const extensions: Extension[] = [
    baseTheme,
    highlightActiveLine(),
    markdown({ markdownLanguage }),
    mathExtension(),
    wrapCompartment.of(getWrapExtension(opts.wrap)),
    tabSizeCompartment.of(getTabSizeExtension(opts.tabSize)),
    typewriterCompartment.of(createTypewriterExtension(opts.typewriter)),
    lineNumbersCompartment.of(getLineNumbersExtension(opts.lineNumbers ?? false)),
    keymap.of([standardKeymap, indentWithTab]),
    EditorState.allowMultipleSelections.of(true),
  ];

  if (opts.updateListener) {
    extensions.push(opts.updateListener);
  }

  return EditorState.create({
    doc: opts.content,
    extensions,
  });
}
