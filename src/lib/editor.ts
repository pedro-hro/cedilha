// Builds a fully-configured CodeMirror 6 editor state for Cedilha.
//
// Composition: Markdown language support + permanent word wrap + typewriter
// scrolling + keyboard navigation, with reactive Compartments for runtime settings.

import { EditorState, type Extension } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { standardKeymap, indentWithTab } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { highlightActiveLine } from "@codemirror/view";
import {
  type EditorOptions,
  tabSizeCompartment,
  typewriterCompartment,
  lineNumbersCompartment,
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
  ".cm-scroller": {
    fontFamily: "var(--font-mono)",
    lineHeight: "1.65",
  },
  ".cm-content": {
    caretColor: "var(--accent)",
    fontFamily: "var(--font-mono)",
    lineHeight: "1.65",
  },
  ".cm-line": {
    lineHeight: "1.65",
    padding: "0 4px",
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
    border: "none",
    marginRight: "16px",
    userSelect: "none",
    minHeight: "0 !important",
    height: "fit-content !important",
    alignSelf: "flex-start",
  },
  ".cm-lineNumbers": {
    borderRight: "1px solid var(--border)",
    paddingRight: "8px",
    minHeight: "0 !important",
    height: "fit-content !important",
  },
  ".cm-lineNumbers .cm-gutterElement": {
    fontFamily: "var(--font-mono)",
    lineHeight: "1.65",
    padding: "0 4px",
    minWidth: "28px",
    textAlign: "right",
    color: "var(--fg-dim)",
  },
  ".cm-activeLine": {
    backgroundColor: "var(--active-line, rgba(255, 255, 255, 0.035))",
    borderRadius: "var(--radius)",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "transparent",
    color: "var(--accent)",
    fontWeight: "600",
  },
});

export function createEditor(opts: FullEditorOptions): EditorState {
  const extensions: Extension[] = [
    baseTheme,
    highlightActiveLine(),
    markdown({ markdownLanguage }),
    // Fixed, opinionated word-wrapping
    EditorView.lineWrapping,
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
